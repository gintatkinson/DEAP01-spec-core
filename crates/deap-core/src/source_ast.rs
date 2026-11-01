//! Zero-regex source-code token-stream lexer, recursive-descent AST extractor, and SysML v2 lowering engine.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C software
//! architecture and bidirectional traceability requirements (FEAT-42). Provides a deterministic,
//! zero-regex parser that extracts structural and behavioral declarations from Rust and Dart
//! source files and lowers them into canonical SysML v2 `PackageDef` models.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Zero-Regex Lexical Scanning: All tokenization, delimiter balancing, and declaration parsing
//!   operate exclusively via character iterators and non-panicking `.get(..)` string slices.
//! - Fail-Closed Syntax Verification: Unclosed braces, parentheses, brackets, or string/comment
//!   literals halt parsing with explicit 1-indexed `file:line:col` diagnostics.
//! - Deterministic M2 Lowering: Source classes/structs map to `PartDef`, fields to `AttributeDef`,
//!   callbacks/streams to `PortDef`, methods/functions to `ActionDef`, enums/states to `StateDef`,
//!   assertions to `ConstraintDef`, and `Realises: [...]` tags to `satisfied_requirements`.
//!
//! ## 3. Preconditions
//! - Source inputs are UTF-8 strings or readable workspace directories containing `.rs` or `.dart` files.
//!
//! ## 4. Postconditions
//! - Returns strongly typed `Vec<SourceAstItem>` or lowered `PackageDef` instances preserving
//!   declaration order, parameter signatures, and requirement realization bindings.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N \log N)$ across directory trees of total byte size $N$; $O(L)$ per file of length $L$.
//! - Space Complexity: $O(N)$ proportional to extracted AST items and lowered SysML v2 entities.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Invalid language flags, missing directories, or unreadable files return `Err(String)`.
//! - State: Per-file parser state machines are isolated with zero global mutable state.
//! - Action: Malformed token streams or unbalanced delimiters fail closed with line/column coordinates.
//! - Resource: Bounded single-pass scanning with zero recursion on delimiter depth checks.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-FEAT-42-SOURCE-AST], [REQ-0608], [DO-330-TQL-4-SOURCE-AST]

#![forbid(unsafe_code)]

use crate::error::CoreError;
use crate::sysml_ast::{
    ActionDef, AttributeDef, ConstraintDef, PackageDef, PartDef, PortDef, RequirementDef, StateDef,
};
use crate::workspace::{
    verify_pinned_cst_dependency_graph, LockManifestSnapshot, OfflineLockGovernanceError,
    MAX_AST_DEPTH,
};
use ra_ap_syntax::AstNode;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Supported source-code language selectors for AST ingestion and docstring synthesis.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and FEAT-42 Section 2.1 CLI language selection rules (`SourceLanguage`).
/// Governs deterministic dispatch between the Rust and Dart recursive-descent AST extractors.
///
/// ## 2. Invariants & Mathematical Properties
/// - Closed 3-variant domain (`Auto`, `Rust`, `Dart`) corresponding to `--language <auto|rust|dart>`.
///
/// ## 3. Preconditions
/// - Constructed via `SourceLanguage::from_flag` or `SourceLanguage::detect_from_extension`.
///
/// ## 4. Postconditions
/// - Drives file extension filtering and language-specific lexical/syntactic rules.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) discriminant comparison.
/// - Space Complexity: O(1) stack value representation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Unknown language flags are rejected by `SourceLanguage::from_flag` with `Err(String)`.
/// - State: Immutable copy type.
/// - Action: Exhaustive pattern matching across all 3 variants.
/// - Resource: Zero heap allocations.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-SOURCE-LANGUAGE-ENUM]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum SourceLanguage {
    /// Automatically detect `.rs` and `.dart` files from file extensions.
    #[default]
    Auto,
    /// Parse Rust (`.rs`) source files exclusively.
    Rust,
    /// Parse Dart (`.dart`) source files exclusively.
    Dart,
}

impl SourceLanguage {
    /// Parses a CLI `--language` flag string (`auto`, `rust`, `dart`) into a `SourceLanguage`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A input validation for `SourceLanguage::from_flag`.
    /// Prevents unrecognized language identifiers from silently bypassing source AST ingestion.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Case-insensitive ASCII matching over `"auto"`, `"rust"`/`"rs"`, and `"dart"`.
    /// - Any other input string returns `Err(String)`.
    ///
    /// ## 3. Preconditions
    /// - `flag: &str` is the user-supplied `--language` CLI option value.
    ///
    /// ## 4. Postconditions
    /// - Returns `Ok(SourceLanguage)` for valid language names or `Err(String)` on invalid input.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(K) where K is `flag.len()`.
    /// - Space Complexity: O(K) for the normalized lowercase string.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Rejects unsupported language strings with a descriptive diagnostic message.
    /// - State: Pure function with no side effects.
    /// - Action: Deterministic branch evaluation.
    /// - Resource: Single bounded string allocation.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-FEAT-42-SOURCE-LANGUAGE-FROM-FLAG]
    pub fn from_flag(flag: &str) -> Result<Self, CoreError> {
        let normalized = flag.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "auto" | "" => Ok(Self::Auto),
            "rust" | "rs" => Ok(Self::Rust),
            "dart" => Ok(Self::Dart),
            other => Err(CoreError::validation(format!(
                "Unsupported source language '{}'. Expected one of: auto, rust, dart.",
                other
            ))),
        }
    }

    /// Detects `SourceLanguage::Rust` or `SourceLanguage::Dart` from a file path extension.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A deterministic file classification (`detect_from_extension`).
    /// Maps `.rs` and `.dart` extensions to their authoritative language parser without heuristics.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Returns `Some(SourceLanguage::Rust)` iff extension is `"rs"` (case-insensitive).
    /// - Returns `Some(SourceLanguage::Dart)` iff extension is `"dart"` (case-insensitive).
    /// - Returns `None` for all other extensions or extensionless paths.
    ///
    /// ## 3. Preconditions
    /// - `path: &Path` is a candidate filesystem path.
    ///
    /// ## 4. Postconditions
    /// - Returns the concrete language variant for `.rs`/`.dart` files, or `None`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(E) where E is the byte length of the file extension.
    /// - Space Complexity: O(1) zero heap allocation.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Safely handles paths without filenames or non-UTF-8 extensions via `and_then`.
    /// - State: Stateless path inspection.
    /// - Action: Never panics.
    /// - Resource: Zero dynamic memory usage.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-FEAT-42-SOURCE-LANGUAGE-DETECT-EXT]
    pub fn detect_from_extension(path: &Path) -> Option<Self> {
        let ext = path.extension().and_then(|e| e.to_str())?;
        if ext.eq_ignore_ascii_case("rs") {
            Some(Self::Rust)
        } else if ext.eq_ignore_ascii_case("dart") {
            Some(Self::Dart)
        } else {
            None
        }
    }

    /// Returns the canonical lowercase string label for `self`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A diagnostic traceability in `SourceLanguage::label`.
    /// Supplies a static human-readable identifier for CLI reporting and error messages.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Bijective mapping from `{Auto, Rust, Dart}` to `{"auto", "rust", "dart"}`.
    ///
    /// ## 3. Preconditions
    /// - `self: SourceLanguage` is a valid initialized enum value.
    ///
    /// ## 4. Postconditions
    /// - Returns a non-empty `'static` string slice naming the language mode.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(1) constant time.
    /// - Space Complexity: O(1) static memory.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Infallible `const fn`.
    /// - State: Pure projection.
    /// - Action: Total match over all enum variants.
    /// - Resource: Zero allocations.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-FEAT-42-SOURCE-LANGUAGE-LABEL]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Rust => "rust",
            Self::Dart => "dart",
        }
    }
}

/// Grammatical category of an extracted Rust or Dart source declaration.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and FEAT-42 Section 3.1 metamodel mapping rules (`SourceItemKind`).
/// Distinguishes structural classifiers (`Struct`, `Class`, `Enum`, `Trait`, `Mixin`, `Extension`, `Typedef`)
/// from behavioral callables (`Function`, `Method`, `Constructor`).
///
/// ## 2. Invariants & Mathematical Properties
/// - Every extracted `SourceAstItem` carries an explicit `SourceItemKind` discriminant.
/// - `is_callable()` holds iff the discriminant is `Function`, `Method`, or `Constructor`.
///
/// ## 3. Preconditions
/// - Assigned by `parse_rust_source_ast` or `parse_dart_source_ast` during declaration classification.
///
/// ## 4. Postconditions
/// - Drives M2 lowering into `PartDef`, `StateDef`, or `ActionDef` and docstring section interpolation.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) discriminant operations.
/// - Space Complexity: O(1) single-byte enum representation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Closed enum prevents uncategorized AST items.
/// - State: Immutable value type.
/// - Action: Exhaustive compile-time matching.
/// - Resource: Zero heap overhead.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-SOURCE-ITEM-KIND]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SourceItemKind {
    /// Rust `struct` declaration.
    Struct,
    /// Rust or Dart `enum` declaration.
    Enum,
    /// Rust `trait` declaration.
    Trait,
    /// Dart `class` declaration.
    Class,
    /// Dart `mixin` declaration.
    Mixin,
    /// Dart `extension` declaration.
    Extension,
    /// Dart `typedef` declaration.
    Typedef,
    /// Top-level Rust `fn` or Dart function declaration.
    Function,
    /// Associated method inside a Rust `impl`/`trait` block or Dart `class`/`mixin`/`extension`.
    Method,
    /// Dart class or factory constructor declaration.
    Constructor,
}

impl SourceItemKind {
    /// Returns the canonical source keyword label for `self`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A diagnostic and docstring synthesis clarity (`SourceItemKind::keyword`).
    /// Provides the exact source construct keyword interpolated into Section 1 of synthesized docstrings.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Maps every `SourceItemKind` variant to a non-empty `'static` keyword string.
    ///
    /// ## 3. Preconditions
    /// - `self: SourceItemKind` is a valid enum discriminant.
    ///
    /// ## 4. Postconditions
    /// - Returns one of `"struct"`, `"enum"`, `"trait"`, `"class"`, `"mixin"`, `"extension"`,
    ///   `"typedef"`, `"fn"`, `"method"`, or `"constructor"`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(1) constant branch dispatch.
    /// - Space Complexity: O(1) static slice return.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Infallible `const fn`.
    /// - State: Zero state mutation.
    /// - Action: Cannot panic.
    /// - Resource: Zero heap allocation.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-FEAT-42-SOURCE-ITEM-KEYWORD]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Struct => "struct",
            Self::Enum => "enum",
            Self::Trait => "trait",
            Self::Class => "class",
            Self::Mixin => "mixin",
            Self::Extension => "extension",
            Self::Typedef => "typedef",
            Self::Function => "fn",
            Self::Method => "method",
            Self::Constructor => "constructor",
        }
    }

    /// Returns `true` if `self` represents a callable function, method, or constructor.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and Check 34/36 per-function uniqueness rules (`is_callable`).
    /// Identifies items that lower to `ActionDef` and require parameter/return-type docstring interpolation.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Evaluates to `true` iff `self` is `Function`, `Method`, or `Constructor`.
    ///
    /// ## 3. Preconditions
    /// - `self: SourceItemKind` is an initialized discriminant.
    ///
    /// ## 4. Postconditions
    /// - Returns a deterministic boolean predicate value.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(1) constant time.
    /// - Space Complexity: O(1) zero allocation.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Infallible `const fn`.
    /// - State: Pure predicate.
    /// - Action: No branching side effects.
    /// - Resource: O(1) stack usage.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-FEAT-42-SOURCE-ITEM-IS-CALLABLE]
    pub const fn is_callable(self) -> bool {
        matches!(self, Self::Function | Self::Method | Self::Constructor)
    }
}

/// Formal input parameter extracted from a Rust or Dart callable signature.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A interface boundary verification (`SourceParam`).
/// Captures formal parameter names and types for `ActionDef::in_params` lowering and Section 3
/// precondition synthesis.
///
/// ## 2. Invariants & Mathematical Properties
/// - `name` and `type_name` are non-empty trimmed strings when extracted from a well-formed parameter list.
///
/// ## 3. Preconditions
/// - Constructed by `parse_rust_param_list` or `parse_dart_param_list`.
///
/// ## 4. Postconditions
/// - Preserves exact parameter identifier and source type annotation for downstream lowering.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(P) where P is the parameter token length.
/// - Space Complexity: O(P) heap storage for `name` and `type_name`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Defaults missing type annotations in untyped Dart parameters to `"Object"`.
/// - State: Value struct with `Clone` and `PartialEq`.
/// - Action: Infallible field access.
/// - Resource: Bounded by declaration signature length.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-SOURCE-PARAM]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceParam {
    /// Formal parameter identifier.
    pub name: String,
    /// Declared source-language type expression.
    pub type_name: String,
}

/// Structural field or callback property extracted from a Rust `struct` or Dart `class`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and FEAT-42 Section 3.1 structural attribute and port mapping (`SourceField`).
/// Distinguishes stateful data attributes (`AttributeDef`) from asynchronous event/callback ports (`PortDef`).
///
/// ## 2. Invariants & Mathematical Properties
/// - When `is_callback_or_port == true`, the field lowers to a `PortDef` on the owning `PartDef`.
/// - When `is_callback_or_port == false`, the field lowers to an `AttributeDef` on the owning `PartDef`.
///
/// ## 3. Preconditions
/// - Extracted from `brace_depth == 1` inside a Rust `struct` or Dart `class`/`State` body.
///
/// ## 4. Postconditions
/// - Retains field identifier, raw type expression, port classification flag, and optional doc comment.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(F) where F is the field declaration line length.
/// - Space Complexity: O(F) owned string fields.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Non-field lines inside struct/class bodies are safely skipped.
/// - State: Value struct with deterministic equality semantics.
/// - Action: No side effects.
/// - Resource: Linear memory in field count.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-SOURCE-FIELD]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceField {
    /// Field or property identifier.
    pub name: String,
    /// Declared source-language type expression.
    pub type_name: String,
    /// True when the field represents an event callback, handler, or stream port (`PortDef`).
    pub is_callback_or_port: bool,
    /// Optional documentation comment attached to the field.
    pub doc: Option<String>,
}

/// Extracted AST node representing a single Rust or Dart type or callable declaration.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and FEAT-42 bidirectional AST representation (`SourceAstItem`).
/// Serves as the unified intermediate representation between source-code token streams,
/// canonical SysML v2 `PackageDef` models, and the 7-section safety docstring synthesizer.
///
/// ## 2. Invariants & Mathematical Properties
/// - `line_no` is 1-indexed and points to the exact declaration line in the source buffer.
/// - `doc_lines` holds the contiguous outer `///` lines attached to the item prior to its attributes.
/// - `satisfied_requirements` contains all bracketed requirement IDs parsed from `Realises: [...]` tags.
///
/// ## 3. Preconditions
/// - Produced by `parse_rust_source_ast` or `parse_dart_source_ast` after syntax validation passes.
///
/// ## 4. Postconditions
/// - Provides complete signature, field, invariant, and traceability metadata for M2 lowering and
///   Check 34/36 unique docstring generation.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) field access; O(M) cloning where M is member/parameter count.
/// - Space Complexity: O(M) proportional to fields, parameters, doc lines, and invariants.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: All optional metadata (`owner_name`, `parent_type`, `return_type`) is explicitly typed via `Option`.
/// - State: Self-contained value record.
/// - Action: Deterministic serialization into SysML v2 AST structures.
/// - Resource: Bounded by single declaration complexity.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-SOURCE-AST-ITEM]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAstItem {
    /// Construct kind (`Struct`, `Class`, `Function`, `Method`, etc.).
    pub kind: SourceItemKind,
    /// Unqualified identifier of the declared item.
    pub name: String,
    /// Enclosing classifier name for `impl` methods or class members.
    pub owner_name: Option<String>,
    /// Optional superclass or implemented trait name (e.g. `StatefulWidget`, `State<MyWidget>`).
    pub parent_type: Option<String>,
    /// 1-indexed line number of the declaration in the source file.
    pub line_no: usize,
    /// Contiguous `///` doc lines preceding the declaration.
    pub doc_lines: Vec<String>,
    /// Structural fields and callback properties (for `Struct` and `Class` items).
    pub fields: Vec<SourceField>,
    /// Formal input parameters (for `Function`, `Method`, and `Constructor` items).
    pub params: Vec<SourceParam>,
    /// Declared return type for callables, if any.
    pub return_type: Option<String>,
    /// Requirement IDs extracted from `Realises: [...]` tags in `doc_lines`.
    pub satisfied_requirements: Vec<String>,
    /// Formal constraint expressions extracted from `assert!` / `assert(...)` statements.
    pub invariants: Vec<String>,
    /// Enum variants or state identifiers associated with the item.
    pub states: Vec<String>,
}

/// Validates lexical and delimiter balance (`{}`, `()`, `[]`, strings, comments) for `source` by auto-detecting language from `file_label`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and FEAT-42 AC-05 negative syntax error recovery (`validate_source_syntax`).
/// Detects unclosed braces, mismatched delimiters, and unterminated literals with exact 1-indexed
/// `line:col` provenance before AST extraction begins.
///
/// ## 2. Invariants & Mathematical Properties
/// - Zero-Regex Token Scanning: Processes bytes/chars sequentially while tracking line and column offsets.
/// - Returns `Ok(())` iff all comments, string literals, and `{}`, `()`, `[]` delimiters are balanced.
///
/// ## 3. Preconditions
/// - `source: &str` is a UTF-8 source buffer.
/// - `file_label: &str` is the file path or diagnostic label used to infer language rules.
///
/// ## 4. Postconditions
/// - Returns `Ok(())` when `source` is lexically and structurally balanced, or `Err(String)` with
///   `file_label:line:col` diagnostic details when malformed.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) where N is `source.len()`.
/// - Space Complexity: O(D) where D is the maximum delimiter nesting depth.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Defaults to `SourceLanguage::Rust` lexical rules when `file_label` has no `.dart` extension.
/// - State: Local delimiter stack with zero shared state.
/// - Action: Halts closed on the first mismatched or unclosed delimiter without panicking.
/// - Resource: Linear scan bounded by input buffer length.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-VALIDATE-SOURCE-SYNTAX]
pub fn validate_source_syntax(source: &str, file_label: &str) -> Result<(), CoreError> {
    let lang = SourceLanguage::detect_from_extension(Path::new(file_label))
        .unwrap_or(SourceLanguage::Rust);
    validate_source_syntax_for_language(source, file_label, lang)
}

/// Active string-literal lexical state tuple `(quote_byte, is_triple_quoted, is_raw_string, open_line, open_col)`.
///
/// # Summary
/// Encapsulates the delimiter byte, triple-quote mode, raw-string mode, and 1-indexed opening `(line, col)`
/// coordinates for an in-flight string literal during source syntax validation.
///
/// # Preconditions
/// - `quote_byte` is `b'"'` or `b'\''`.
/// - `open_line >= 1` and `open_col >= 1`.
///
/// # Postconditions
/// - Preserves exact opening coordinates for fail-closed unclosed-string diagnostics and interpolation restoration.
///
/// # Invariants
/// - Copy-semantic 5-tuple with zero heap allocation.
///
/// # Failure Modes
/// - None (pure type alias).
///
/// # Side Effects
/// - None.
///
/// # Determinism
/// - Deterministic value layout across all target architectures.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and Check 34 zero-suppression rules (`ActiveStringState`).
/// Eliminates complex inline tuple types in `validate_source_syntax_for_language` without lint-suppression attributes.
///
/// ## 2. Invariants & Mathematical Properties
/// - Layout: `(u8, bool, bool, usize, usize)` representing `(quote, is_triple, is_raw, open_line, open_col)`.
///
/// ## 3. Preconditions
/// - Instantiated only when entering a single-, double-, or triple-quoted string literal.
///
/// ## 4. Postconditions
/// - Restored from `InterpolationFrame` when a nested `${...}` expression closes.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) copy access.
/// - Space Complexity: O(1) stack slot.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Total value type; cannot fail or panic.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-ACTIVE-STRING-STATE]
type ActiveStringState = (u8, bool, bool, usize, usize);

/// Saved string-interpolation stack frame `(saved_string_state, outer_delimiter_stack_depth)`.
///
/// # Summary
/// Records the enclosing [`ActiveStringState`] and delimiter stack depth prior to entering a `${...}`
/// string interpolation block so string scanning resumes accurately when the matching `}` is popped.
///
/// # Preconditions
/// - Pushed onto `interp_stack` immediately before pushing the interpolation `'{'` delimiter onto `stack`.
///
/// # Postconditions
/// - Restores `active_str` to `saved_string_state` iff `stack.len()` returns to `outer_delimiter_stack_depth`.
///
/// # Invariants
/// - Preserves arbitrary nesting depth of interpolated strings and inner braced expressions.
///
/// # Failure Modes
/// - None (pure type alias).
///
/// # Side Effects
/// - None.
///
/// # Determinism
/// - Deterministic stack frame representation.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and Check 34 zero-suppression rules (`InterpolationFrame`).
/// Replaces nested tuple type annotations in `validate_source_syntax_for_language` without lint-suppression attributes.
///
/// ## 2. Invariants & Mathematical Properties
/// - Pairs an [`ActiveStringState`] with the `usize` delimiter stack depth at `${` entry.
///
/// ## 3. Preconditions
/// - Constructed only inside Dart non-raw string interpolation `${`.
///
/// ## 4. Postconditions
/// - Popped when the matching `}` restores `stack.len()` to the recorded depth.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) push/pop.
/// - Space Complexity: O(1) per interpolation nesting level.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Total value type; cannot fail or panic.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-INTERPOLATION-FRAME]
type SyntaxInterpolationFrame = (ActiveStringState, usize);

/// Unified string interpolation stack frame for `REQ-0605` line-by-line Rust and Dart lexical scanning.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for `InterpolationFrame` (`REQ-0605` `AC-01`).
///
/// ## 2. Invariants & Mathematical Properties
/// - Records the enclosing string literal's `quote_byte` (`b'"'` or `b'\''`), `is_triple`, `is_raw`, and active `brace_depth` inside `${...}`.
/// - Pushed when entering `${` inside a non-raw string literal and popped when `brace_depth` returns to `0`.
///
/// ## 3. Preconditions
/// - Constructed when encountering `${` inside an active non-raw string literal in `scan_line_with_unified_lex_state`.
///
/// ## 4. Postconditions
/// - Restores `(quote_byte, is_triple, is_raw)` to `RustDartLexState::active_string` upon matching `}`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) construction and field access.
/// - Space Complexity: O(1) stack value representation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Plain value struct with `Copy` and `Eq` semantics.
/// - State: Immutable once pushed except for bounded `brace_depth` updates.
/// - Action: Zero panics or allocations.
/// - Resource: Constant stack footprint.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0605], [REQ-FEAT-42-INTERPOLATION-FRAME]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InterpolationFrame {
    /// Quote delimiter byte (`b'"'` or `b'\''`) of the suspended outer string literal.
    pub quote_byte: u8,
    /// Whether the suspended outer string literal is triple-quoted (`"""` or `'''`).
    pub is_triple: bool,
    /// Whether the suspended outer string literal is a raw string.
    pub is_raw: bool,
    /// Current nesting depth of `{` ... `}` inside the active `${...}` interpolation expression.
    pub brace_depth: usize,
}

/// Unified stateful lexical scanner context across Rust and Dart source lines (`REQ-0605` `AC-01`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for `RustDartLexState` (`REQ-0605` `AC-01..AC-02`).
///
/// ## 2. Invariants & Mathematical Properties
/// - Tracks `language`, nested `block_comment_depth`, Rust hashed raw string `raw_hash_count`, active string tuple `(quote_byte, is_triple, is_raw)`, and nested `interp_stack` across line boundaries.
///
/// ## 3. Preconditions
/// - Initialized via `RustDartLexState::default()` (or with explicit `language: SourceLanguage::Rust` / `SourceLanguage::Dart`) before scanning line 1 of a source file.
///
/// ## 4. Postconditions
/// - Updated in place by [`scan_line_with_unified_lex_state`] on each source line.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) state transitions per scanned token.
/// - Space Complexity: O(D) where D is the maximum `${...}` interpolation nesting depth.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Safe value struct with `Default`, `Clone`, and `PartialEq`/`Eq`.
/// - State: Isolated per-file lexical state machine.
/// - Action: Non-panicking saturating counter updates.
/// - Resource: Bounded vector for nested string interpolations.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0605]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RustDartLexState {
    /// Target source language (`Auto`, `Rust`, or `Dart`).
    pub language: SourceLanguage,
    /// Nesting depth of active `/* ... */` block comments (`0` when outside block comments).
    pub block_comment_depth: usize,
    /// Active Rust raw string `#` delimiter count (`Some(hashes)` inside `r#"..."#`, `None` otherwise).
    pub raw_hash_count: Option<usize>,
    /// Active string literal state `(quote_byte, is_triple, is_raw)` when inside a string literal.
    pub active_string: Option<(u8, bool, bool)>,
    /// Stack of suspended outer string literals across nested `${...}` interpolations.
    pub interp_stack: Vec<InterpolationFrame>,
}

/// Scoped declaration and test/annotation gate state across Rust and Dart AST/audit passes (`REQ-0605` `AC-02`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for `ScopedItemScannerState` (`REQ-0605` `AC-02`).
///
/// ## 2. Invariants & Mathematical Properties
/// - Tracks `mod_tests_brace_depth`, `cfg_test_block_brace_depth`, `cfg_test_pending`, and `dart_annotation_paren_depth` so multi-line `@pragma(\n...\n)` annotations and `#[cfg(test)]` blocks are tracked deterministically.
///
/// ## 3. Preconditions
/// - Initialized via `ScopedItemScannerState::default()` at the beginning of a file scan.
///
/// ## 4. Postconditions
/// - Preserves accumulated `///` documentation across multi-line Dart annotations while `dart_annotation_paren_depth > 0`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) field reads and updates.
/// - Space Complexity: O(1) stack footprint.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Total value struct with `Default`, `Clone`, and `PartialEq`/`Eq`.
/// - State: Isolated per-file scope state.
/// - Action: Zero panics or heap allocations.
/// - Resource: Constant stack memory.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0605]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScopedItemScannerState {
    /// Brace depth at which a `mod tests` block was entered (`None` when outside `mod tests`).
    pub mod_tests_brace_depth: Option<usize>,
    /// Brace depth at which a `#[cfg(test)]` item block was entered (`None` when outside).
    pub cfg_test_block_brace_depth: Option<usize>,
    /// Whether a `#[cfg(test)]` attribute is pending attachment to the next item declaration.
    pub cfg_test_pending: bool,
    /// Open parenthesis depth inside a multi-line Dart `@annotation(...)` prefix (`0` when outside).
    pub dart_annotation_paren_depth: usize,
}

/// Extracted method signature `(in_parameters, out_parameters, return_type_name)`.
/// # Summary
/// Captures normalized input parameter `(name, type)` pairs, output parameter `(name, type)` pairs, and optional return type name.
/// # Preconditions
/// - Constructed from syntactically valid source AST callable declarations via [`extract_method_signature`].
/// # Postconditions
/// - Preserves exact parameter declaration order, SysML-normalized output parameter list, and optional return type metadata.
/// # Invariants
/// - Tuple layout is `(Vec<(String, String)>, Vec<(String, String)>, Option<String>)`.
/// # Failure Modes
/// - None (pure type alias).
/// # Side Effects
/// - None.
/// # Determinism
/// - Deterministic value layout across all target architectures.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, Blueprint 01 Section 6, and Check 34 zero-suppression rules (`ExtractedMethodSignature`).
/// ## 2. Invariants & Mathematical Properties
/// - Replaces complex nested tuple annotations in method extraction tables without lint suppressions.
/// ## 3. Preconditions
/// - Instantiated during AST callable signature extraction (`extract_method_signature`).
/// ## 4. Postconditions
/// - Stored inside [`ComponentMethodTable`] keyed by owning component name and consumed during SysML action lowering.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) construction. Space Complexity: O(P) for P parameters.
/// ## 6. Failure Modes & Fault Tolerance
/// - Total value type; cannot fail or panic.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0601], [REQ-FEAT-42-EXTRACTED-METHOD-SIG]
pub type ExtractedMethodSignature = (Vec<(String, String)>, Vec<(String, String)>, Option<String>);

/// Deterministic lookup table mapping component names to extracted method signatures.
/// # Summary
/// Maps each component or type identifier to its ordered vector of [`ExtractedMethodSignature`] entries.
/// # Preconditions
/// - Keys are non-empty sanitized component or struct identifiers parsed from source ASTs.
/// # Postconditions
/// - Maintains lexicographically sorted component keys via `BTreeMap`.
/// # Invariants
/// - Every entry maps `String -> Vec<ExtractedMethodSignature>`.
/// # Failure Modes
/// - None (pure type alias).
/// # Side Effects
/// - None.
/// # Determinism
/// - Deterministic iteration order guaranteed by `std::collections::BTreeMap`.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, Blueprint 01 Section 6, and Check 34 zero-suppression rules (`ComponentMethodTable`).
/// ## 2. Invariants & Mathematical Properties
/// - Eliminates `clippy::type_complexity` on nested method table signatures via semantic aliasing.
/// ## 3. Preconditions
/// - Populated during component and impl block AST traversal via [`extract_component_method_table`].
/// ## 4. Postconditions
/// - Provides O(log N) lookup by component name without raw indexing.
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(log N) lookup/insertion. Space Complexity: O(N * M) across N components.
/// ## 6. Failure Modes & Fault Tolerance
/// - Total collection type alias; lookups return `Option` without panicking.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0601], [REQ-FEAT-42-COMPONENT-METHOD-TABLE]
pub type ComponentMethodTable = std::collections::BTreeMap<String, Vec<ExtractedMethodSignature>>;

/// Extracts a normalized SysML v2 [`ExtractedMethodSignature`] from a callable [`SourceAstItem`].
///
/// # Summary
/// Converts a callable AST item (`Function`, `Method`, or `Constructor`) into `(in_parameters, out_parameters, return_type_name)`
/// with SysML-sanitized parameter names and normalized type identifiers, returning `None` for non-callable AST items.
///
/// # Preconditions
/// - `item` is a borrowed [`SourceAstItem`] produced by a language-specific source AST parser.
///
/// # Postconditions
/// - Returns `None` when `!item.kind.is_callable()`.
/// - Returns `Some((in_parameters, out_parameters, return_type_name))` when `item.kind.is_callable()`.
///
/// # Invariants
/// - Every input parameter pair has a non-empty sanitized identifier (`"param"` fallback) and SysML-normalized type name.
/// - Non-unit, non-void return types yield a single `("result", normalized_type)` entry in `out_parameters` and matching `return_type_name`.
///
/// # Failure Modes
/// - Non-callable items (`Struct`, `Enum`, `Trait`, `Widget`) return `None` without panicking.
///
/// # Side Effects
/// - Pure transformation; performs zero I/O and mutates no external state.
///
/// # Determinism
/// - Deterministic output for identical `SourceAstItem` inputs.
///
/// # Parameters
/// - `item`: Borrowed [`SourceAstItem`] to inspect and normalize.
///
/// # Returns
/// - `Option<ExtractedMethodSignature>` containing `(in_parameters, out_parameters, return_type_name)`.
///
/// # Errors
/// - None (total option-returning projection).
///
/// # Panics
/// - Never panics.
///
/// # Safety
/// - Safe pure Rust implementation (`#![forbid(unsafe_code)]` compliant).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, Blueprint 01 Section 6, and Check 34 zero-suppression rules (`extract_method_signature`).
/// Centralizes callable parameter and return-type normalization behind the [`ExtractedMethodSignature`] type alias.
///
/// ## 2. Invariants & Mathematical Properties
/// - `item.kind.is_callable() == false` implies `extract_method_signature(item) == None`.
/// - `return_type_name == out_parameters.first().map(|(_, ty)| ty.clone())`.
///
/// ## 3. Preconditions
/// - `item: &SourceAstItem` is a valid in-memory AST node reference.
///
/// ## 4. Postconditions
/// - Returns a normalized [`ExtractedMethodSignature`] ready for [`ComponentMethodTable`] indexing or SysML `ActionDef` lowering.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(P + R) where P is `item.params.len()` and R is return type byte length.
/// - Space Complexity: O(P + R) for owned parameter and return type strings.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Rejects non-callable `SourceAstItemKind` variants cleanly via `None`.
/// - State: Stateless projection.
/// - Action: Returns `None` on non-callable AST items.
/// - Resource: Bounded by parsed parameter count P.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0601], [REQ-FEAT-42-EXTRACT-METHOD-SIGNATURE]
pub fn extract_method_signature(item: &SourceAstItem) -> Option<ExtractedMethodSignature> {
    if !item.kind.is_callable() {
        return None;
    }
    let in_parameters: Vec<(String, String)> = item
        .params
        .iter()
        .map(|p| {
            (
                sanitize_sysml_identifier(&p.name, "param"),
                normalize_type_for_sysml(&p.type_name),
            )
        })
        .collect();
    let out_parameters: Vec<(String, String)> = match item.return_type.as_deref().map(str::trim) {
        Some(trimmed_ret)
            if !trimmed_ret.is_empty() && trimmed_ret != "()" && trimmed_ret != "void" =>
        {
            vec![("result".to_string(), normalize_type_for_sysml(trimmed_ret))]
        }
        _ => Vec::new(),
    };
    let return_type_name = out_parameters.first().map(|(_, ty)| ty.clone());
    Some((in_parameters, out_parameters, return_type_name))
}

/// Builds a deterministic [`ComponentMethodTable`] from a slice of parsed [`SourceAstItem`] nodes.
///
/// # Summary
/// Iterates over `items`, extracts each callable signature via [`extract_method_signature`], resolves the sanitized owning
/// component key (`"TopLevel"` when `owner_name` is `None`), and groups signatures into a lexicographically ordered `BTreeMap`.
///
/// # Preconditions
/// - `items` is a slice of [`SourceAstItem`] declarations extracted from source files.
///
/// # Postconditions
/// - Returns a [`ComponentMethodTable`] mapping each sanitized component identifier to its declaration-ordered method signatures.
///
/// # Invariants
/// - Non-callable items are skipped via [`extract_method_signature`].
/// - Key order is strictly deterministic (`BTreeMap<String, Vec<ExtractedMethodSignature>>`).
///
/// # Failure Modes
/// - Empty slices or slices without callable items return an empty `ComponentMethodTable` without panicking.
///
/// # Side Effects
/// - Pure aggregation; performs zero I/O and mutates no external state.
///
/// # Determinism
/// - Deterministic map construction and per-component vector ordering.
///
/// # Parameters
/// - `items`: Slice of [`SourceAstItem`] declarations to index.
///
/// # Returns
/// - [`ComponentMethodTable`] keyed by sanitized component identifier or `"TopLevel"`.
///
/// # Errors
/// - None.
///
/// # Panics
/// - Never panics.
///
/// # Safety
/// - Safe pure Rust implementation (`#![forbid(unsafe_code)]` compliant).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, Blueprint 01 Section 6, and Check 34 zero-suppression rules (`extract_component_method_table`).
/// Aggregates component method signatures into [`ComponentMethodTable`] without nested tuple lint suppressions.
///
/// ## 2. Invariants & Mathematical Properties
/// - Total number of stored [`ExtractedMethodSignature`] entries equals the count of `item` elements where `item.kind.is_callable()`.
///
/// ## 3. Preconditions
/// - `items: &[SourceAstItem]` is a valid slice reference.
///
/// ## 4. Postconditions
/// - Every callable item is indexed under `sanitize_sysml_identifier(owner, "ComponentPart")` or `"TopLevel"`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N * (P + log C)) for N AST items, P parameters per callable, and C unique component owners.
/// - Space Complexity: O(N * P) for the populated [`ComponentMethodTable`].
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Accepts empty or non-callable slices gracefully.
/// - State: Local `BTreeMap` accumulator only.
/// - Action: Skips non-callable items via `let Some(sig) = extract_method_signature(item) else { continue; }`.
/// - Resource: Bounded by input slice length `items.len()`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0601], [REQ-FEAT-42-EXTRACT-COMPONENT-METHOD-TABLE]
pub fn extract_component_method_table(items: &[SourceAstItem]) -> ComponentMethodTable {
    let mut table = ComponentMethodTable::new();
    for item in items {
        let resolved_item = resolve_callable_param_types(item, items);
        let Some(sig) = extract_method_signature(&resolved_item) else {
            continue;
        };
        let owner_key = item
            .owner_name
            .as_deref()
            .map(|o| sanitize_sysml_identifier(o, "ComponentPart"))
            .unwrap_or_else(|| "TopLevel".to_string());
        table.entry(owner_key).or_default().push(sig);
    }
    table
}

/// Validates lexical and delimiter balance (`{}`, `()`, `[]`, strings, comments) under explicit `lang` rules.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and FEAT-42 AC-05 fail-closed parser verification (`validate_source_syntax_for_language`).
/// Distinguishes Rust lifetimes (`'a`, `'static`) and raw strings (`r#"..."#`) from Dart single-quoted
/// and triple-quoted strings while verifying total delimiter balance.
///
/// ## 2. Invariants & Mathematical Properties
/// - Tracks exact 1-indexed `(line, col)` coordinates for every opening delimiter pushed onto `stack`.
/// - Rejects any closing delimiter whose top-of-stack counterpart does not match, and rejects non-empty
///   `stack` or unclosed string/comment state at EOF.
///
/// ## 3. Preconditions
/// - `source: &str` is a UTF-8 source string.
/// - `file_label: &str` is the diagnostic path prefix.
/// - `lang: SourceLanguage` specifies `Rust` or `Dart` lexical conventions.
///
/// ## 4. Postconditions
/// - Returns `Ok(())` on balanced syntax, or `Err(String)` containing `line` and `column` coordinates.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) single-pass scan over `source.as_bytes()`.
/// - Space Complexity: O(D) where D is nesting depth of `{`, `(`, `[`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Non-panicking `.get(..)` byte inspection throughout.
/// - State: Pure local state machine (`stack`, quote flags, comment depth).
/// - Action: Reports exact opening or mismatch location on syntax error.
/// - Resource: Zero recursion; bounded vector stack.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-VALIDATE-SYNTAX-LANG]
pub fn validate_source_syntax_for_language(
    source: &str,
    file_label: &str,
    lang: SourceLanguage,
) -> Result<(), CoreError> {
    let bytes = source.as_bytes();
    let mut idx: usize = 0;
    let mut line: usize = 1;
    let mut col: usize = 1;
    let mut stack: Vec<(u8, usize, usize)> = Vec::new();
    let mut active_str: Option<ActiveStringState> = None;
    let mut interp_stack: Vec<SyntaxInterpolationFrame> = Vec::new();
    let is_dart = lang == SourceLanguage::Dart;

    while let Some(&b) = bytes.get(idx) {
        if let Some((quote, is_triple, is_raw, open_line, open_col)) = active_str {
            if !is_raw && b == b'\\' {
                idx = idx.saturating_add(2);
                col = col.saturating_add(2);
                continue;
            }
            if is_dart && !is_raw && b == b'$' && bytes.get(idx.saturating_add(1)) == Some(&b'{') {
                interp_stack.push(((quote, is_triple, is_raw, open_line, open_col), stack.len()));
                stack.push((b'{', line, col.saturating_add(1)));
                active_str = None;
                idx = idx.saturating_add(2);
                col = col.saturating_add(2);
                continue;
            }
            if is_triple {
                if b == quote
                    && bytes.get(idx.saturating_add(1)) == Some(&quote)
                    && bytes.get(idx.saturating_add(2)) == Some(&quote)
                {
                    active_str = None;
                    idx = idx.saturating_add(3);
                    col = col.saturating_add(3);
                    continue;
                }
            } else if b == quote {
                active_str = None;
                idx = idx.saturating_add(1);
                col = col.saturating_add(1);
                continue;
            }
            if b == b'\n' {
                if quote == b'\'' && !is_triple {
                    return Err(CoreError::syntax(format!(
                        "{}:{}:{}: syntax error: unclosed single-quoted string at line {}, column {}",
                        file_label, open_line, open_col, open_line, open_col
                    )));
                }
                line = line.saturating_add(1);
                col = 1;
                idx = idx.saturating_add(1);
                continue;
            }
            col = col.saturating_add(1);
            idx = idx.saturating_add(1);
            continue;
        }

        if b == b'\n' {
            line = line.saturating_add(1);
            col = 1;
            idx = idx.saturating_add(1);
            continue;
        }

        // Line comment `// ...`
        if b == b'/' && bytes.get(idx.saturating_add(1)) == Some(&b'/') {
            while let Some(&cb) = bytes.get(idx) {
                if cb == b'\n' {
                    break;
                }
                idx = idx.saturating_add(1);
            }
            continue;
        }

        // Block comment `/* ... */` (supports nested `/* ... /* ... */ ... */`)
        if b == b'/' && bytes.get(idx.saturating_add(1)) == Some(&b'*') {
            let open_line = line;
            let open_col = col;
            idx = idx.saturating_add(2);
            col = col.saturating_add(2);
            let mut depth: usize = 1;
            while let Some(&cb) = bytes.get(idx) {
                if cb == b'/' && bytes.get(idx.saturating_add(1)) == Some(&b'*') {
                    depth = depth.saturating_add(1);
                    idx = idx.saturating_add(2);
                    col = col.saturating_add(2);
                    continue;
                }
                if cb == b'*' && bytes.get(idx.saturating_add(1)) == Some(&b'/') {
                    depth = depth.saturating_sub(1);
                    idx = idx.saturating_add(2);
                    col = col.saturating_add(2);
                    if depth == 0 {
                        break;
                    }
                    continue;
                }
                if cb == b'\n' {
                    line = line.saturating_add(1);
                    col = 1;
                } else {
                    col = col.saturating_add(1);
                }
                idx = idx.saturating_add(1);
            }
            if depth != 0 {
                return Err(CoreError::syntax(format!(
                    "{}:{}:{}: syntax error: unclosed block comment starting at line {}, column {}",
                    file_label, open_line, open_col, open_line, open_col
                )));
            }
            continue;
        }

        // Rust raw string literal `r#"..."#` or raw byte string literal `br#"..."#`
        if !is_dart && (b == b'r' || b == b'R') && is_valid_raw_string_prefix(bytes, idx, true) {
            let mut scan = idx.saturating_add(1);
            let mut hashes: usize = 0;
            while bytes.get(scan) == Some(&b'#') {
                hashes = hashes.saturating_add(1);
                scan = scan.saturating_add(1);
            }
            if hashes > 0 && bytes.get(scan) == Some(&b'"') {
                let open_line = line;
                let open_col = col;
                col = col.saturating_add(hashes.saturating_add(2));
                idx = scan.saturating_add(1);
                let mut closed = false;
                while let Some(&rb) = bytes.get(idx) {
                    if rb == b'"' {
                        let mut match_hashes: usize = 0;
                        let mut h_scan = idx.saturating_add(1);
                        while match_hashes < hashes && bytes.get(h_scan) == Some(&b'#') {
                            match_hashes = match_hashes.saturating_add(1);
                            h_scan = h_scan.saturating_add(1);
                        }
                        if match_hashes == hashes {
                            idx = h_scan;
                            col = col.saturating_add(hashes.saturating_add(1));
                            closed = true;
                            break;
                        }
                    }
                    if rb == b'\n' {
                        line = line.saturating_add(1);
                        col = 1;
                    } else {
                        col = col.saturating_add(1);
                    }
                    idx = idx.saturating_add(1);
                }
                if !closed {
                    return Err(CoreError::syntax(format!(
                        "{}:{}:{}: syntax error: unclosed raw string literal at line {}, column {}",
                        file_label, open_line, open_col, open_line, open_col
                    )));
                }
                continue;
            }
        }

        let is_raw_quote =
            idx > 0 && is_valid_raw_string_prefix(bytes, idx.saturating_sub(1), !is_dart);

        // Triple-quoted string in Dart (`"""..."""` or `'''...'''`)
        if is_dart
            && (b == b'"' || b == b'\'')
            && bytes.get(idx.saturating_add(1)) == Some(&b)
            && bytes.get(idx.saturating_add(2)) == Some(&b)
        {
            active_str = Some((b, true, is_raw_quote, line, col));
            idx = idx.saturating_add(3);
            col = col.saturating_add(3);
            continue;
        }

        // Double-quoted string literal `"..."`
        if b == b'"' {
            active_str = Some((b'"', false, is_raw_quote, line, col));
            idx = idx.saturating_add(1);
            col = col.saturating_add(1);
            continue;
        }

        // Single-quoted literal: in Dart `'...'` is a string; in Rust `'c'` is a char or `'a` is a lifetime
        if b == b'\'' {
            if is_dart {
                active_str = Some((b'\'', false, is_raw_quote, line, col));
                idx = idx.saturating_add(1);
                col = col.saturating_add(1);
                continue;
            } else {
                // Rust: check if this is a char literal `'x'`, `'\...'`, or multi-byte UTF-8 char literal
                if let Some(char_len) = source.get(idx..).and_then(single_char_literal_byte_len) {
                    idx = idx.saturating_add(char_len);
                    col = col.saturating_add(char_len);
                    continue;
                }
                // Otherwise it is a Rust lifetime like `'a` or `'static`
            }
        }

        if b == b'{' || b == b'(' || b == b'[' {
            stack.push((b, line, col));
        } else if b == b'}' || b == b')' || b == b']' {
            let expected_open = match b {
                b'}' => b'{',
                b')' => b'(',
                _ => b'[',
            };
            match stack.pop() {
                Some((open_b, open_line, open_col)) if open_b == expected_open => {
                    let _matched = (open_line, open_col);
                    if b == b'}' {
                        if let Some(&(saved_str, saved_stack_len)) = interp_stack.last() {
                            if stack.len() == saved_stack_len {
                                interp_stack.pop();
                                active_str = Some(saved_str);
                            }
                        }
                    }
                }
                Some((open_b, open_line, open_col)) => {
                    return Err(CoreError::syntax(format!(
                        "{}:{}:{}: syntax error: unexpected closing '{}' at line {}, column {} (mismatched with '{}' opened at line {}, column {})",
                        file_label,
                        line,
                        col,
                        char::from(b),
                        line,
                        col,
                        char::from(open_b),
                        open_line,
                        open_col
                    )));
                }
                None => {
                    return Err(CoreError::syntax(format!(
                        "{}:{}:{}: syntax error: unexpected closing '{}' at line {}, column {} with no matching opening delimiter",
                        file_label,
                        line,
                        col,
                        char::from(b),
                        line,
                        col
                    )));
                }
            }
        }

        idx = idx.saturating_add(1);
        col = col.saturating_add(1);
    }

    if let Some((quote, is_triple, _, open_line, open_col)) = active_str {
        let kind_label = if is_triple {
            "triple-quoted string"
        } else if quote == b'\'' {
            "single-quoted string"
        } else {
            "string literal"
        };
        return Err(CoreError::syntax(format!(
            "{}:{}:{}: syntax error: unclosed {} at line {}, column {}",
            file_label, open_line, open_col, kind_label, open_line, open_col
        )));
    }

    if let Some((open_b, open_line, open_col)) = stack.pop() {
        return Err(CoreError::syntax(format!(
            "{}:{}:{}: syntax error: unclosed delimiter '{}' opened at line {}, column {}",
            file_label,
            open_line,
            open_col,
            char::from(open_b),
            open_line,
            open_col
        )));
    }

    Ok(())
}

/// Parses a Rust source string into a declaration-ordered vector of `SourceAstItem` records.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and FEAT-42 AC-02 Rust AST extraction (`parse_rust_source_ast`).
/// Extracts production `struct`, `enum`, `trait`, and `fn` declarations (including `impl` block
/// methods, struct fields, formal parameters, return types, `Realises:` tags, and `assert!` invariants)
/// without regular expressions.
///
/// ## 2. Invariants & Mathematical Properties
/// - Executes `validate_source_syntax_for_language(..., SourceLanguage::Rust)` prior to extraction.
/// - Associates methods inside `impl <Target>` or `impl <Trait> for <Target>` blocks with `owner_name: Some(Target)`.
/// - Stops scanning production items once `#[cfg(test)] mod tests` is entered.
///
/// ## 3. Preconditions
/// - `source: &str` contains UTF-8 Rust source text.
/// - `file_label: &str` is the diagnostic path for syntax error reporting.
///
/// ## 4. Postconditions
/// - Returns `Ok(Vec<SourceAstItem>)` with all discovered Rust items, or `Err(String)` on syntax failure.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(L) where L is the number of lines in `source`.
/// - Space Complexity: O(I) where I is the total count of extracted items, fields, and parameters.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Propagates syntax/delimiter errors from `validate_source_syntax_for_language`.
/// - State: Local line scanner state tracking `brace_depth`, `paren_depth`, and active struct/enum/impl context.
/// - Action: Non-panicking `.get(..)` and prefix stripping on all source slices.
/// - Resource: Linear memory allocation proportional to AST item count.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-PARSE-RUST-AST]
pub fn parse_rust_source_ast(
    source: &str,
    file_label: &str,
) -> Result<Vec<SourceAstItem>, CoreError> {
    validate_source_syntax_for_language(source, file_label, SourceLanguage::Rust)?;

    let mut items: Vec<SourceAstItem> = Vec::new();
    let mut current_doc: Vec<String> = Vec::new();
    let mut rust_lex_state = RustStringScannerState::Normal;
    let mut in_mod_tests = false;
    let mut cfg_test_pending = false;
    let mut attr_bracket_depth: usize = 0;
    let mut brace_depth: usize = 0;
    let mut paren_depth: usize = 0;

    let mut active_struct_idx: Option<usize> = None;
    let mut active_enum_idx: Option<usize> = None;
    let mut active_impl_owner: Option<String> = None;
    let mut pending_impl_header: Option<String> = None;
    let mut active_fn_idx: Option<usize> = None;
    let mut pending_sig_fn_idx: Option<usize> = None;
    let mut pending_sig_text = String::new();

    for (idx, raw_line) in source.lines().enumerate() {
        let line_no = idx.saturating_add(1);
        let trimmed = raw_line.trim();
        let started_in_normal = rust_lex_state == RustStringScannerState::Normal;

        if started_in_normal && trimmed.starts_with("//!") {
            continue;
        }
        if started_in_normal && trimmed.starts_with("///") {
            if !in_mod_tests {
                current_doc.push(trimmed.to_string());
            }
            continue;
        }

        let sanitized_line = scan_rust_line_preserving_lex_state(trimmed, &mut rust_lex_state);
        let code = sanitized_line.trim();
        if code.is_empty()
            || (!started_in_normal && rust_lex_state != RustStringScannerState::Normal)
        {
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

        // Accumulate multi-line function signature if open paren was not closed on the fn line
        if let Some(fn_idx) = pending_sig_fn_idx {
            pending_sig_text.push(' ');
            pending_sig_text.push_str(code);
            if code.contains('{') || code.ends_with(';') {
                let (params, ret_type) = parse_rust_callable_signature(&pending_sig_text);
                if let Some(item) = items.get_mut(fn_idx) {
                    item.params = params;
                    item.return_type = ret_type;
                }
                pending_sig_fn_idx = None;
                pending_sig_text.clear();
            }
        }

        if paren_depth == 0 {
            if brace_depth == 0 {
                let after_unsafe = code.strip_prefix("unsafe ").unwrap_or(code).trim_start();
                let starts_impl = after_unsafe == "impl"
                    || after_unsafe.starts_with("impl<")
                    || after_unsafe.starts_with("impl ");
                if pending_impl_header.is_some() || starts_impl {
                    let mut combined_impl = pending_impl_header.take().unwrap_or_default();
                    if !combined_impl.is_empty() {
                        combined_impl.push(' ');
                    }
                    combined_impl.push_str(code);
                    if code.contains('{') || code.contains(';') || code.contains('}') {
                        if let Some(impl_target) = parse_rust_impl_header(&combined_impl) {
                            active_impl_owner = Some(impl_target);
                        }
                        cfg_test_pending = false;
                        current_doc.clear();
                    } else {
                        pending_impl_header = Some(combined_impl);
                    }
                } else if let Some((kind, name)) = classify_rust_item_line_ast(code) {
                    if cfg_test_pending {
                        cfg_test_pending = false;
                        current_doc.clear();
                    } else {
                        let reqs = extract_realises_ids_from_doc_lines(&current_doc);
                        let (params, return_type) = if kind.is_callable() {
                            parse_rust_callable_signature(code)
                        } else {
                            (Vec::new(), None)
                        };
                        let item_idx = items.len();
                        items.push(SourceAstItem {
                            kind,
                            name: name.clone(),
                            owner_name: if kind == SourceItemKind::Trait {
                                Some(name.clone())
                            } else {
                                None
                            },
                            parent_type: None,
                            line_no,
                            doc_lines: current_doc.clone(),
                            fields: Vec::new(),
                            params,
                            return_type,
                            satisfied_requirements: reqs,
                            invariants: Vec::new(),
                            states: Vec::new(),
                        });
                        if kind == SourceItemKind::Struct
                            && code.contains('{')
                            && !code.contains('}')
                        {
                            active_struct_idx = Some(item_idx);
                        } else if kind == SourceItemKind::Enum
                            && code.contains('{')
                            && !code.contains('}')
                        {
                            active_enum_idx = Some(item_idx);
                        } else if kind == SourceItemKind::Trait
                            && code.contains('{')
                            && !code.contains('}')
                        {
                            active_impl_owner = Some(name);
                        } else if kind.is_callable() {
                            active_fn_idx = Some(item_idx);
                            if !code.contains('{') && !code.ends_with(';') {
                                pending_sig_fn_idx = Some(item_idx);
                                pending_sig_text = code.to_string();
                            }
                        }
                        current_doc.clear();
                    }
                } else {
                    cfg_test_pending = false;
                    current_doc.clear();
                }
            } else if brace_depth == 1 {
                if let Some(s_idx) = active_struct_idx {
                    if let Some(field) = parse_rust_struct_field_line(code, &current_doc) {
                        if let Some(s_item) = items.get_mut(s_idx) {
                            s_item.fields.push(field);
                        }
                    }
                    current_doc.clear();
                } else if let Some(e_idx) = active_enum_idx {
                    if let Some(variant) = parse_enum_variant_line(code) {
                        if let Some(e_item) = items.get_mut(e_idx) {
                            e_item.states.push(variant);
                        }
                    }
                    current_doc.clear();
                } else if let Some((kind, name)) = classify_rust_item_line_ast(code) {
                    if cfg_test_pending {
                        cfg_test_pending = false;
                        current_doc.clear();
                    } else {
                        let actual_kind =
                            if kind == SourceItemKind::Function && active_impl_owner.is_some() {
                                SourceItemKind::Method
                            } else {
                                kind
                            };
                        let reqs = extract_realises_ids_from_doc_lines(&current_doc);
                        let (params, return_type) = if actual_kind.is_callable() {
                            parse_rust_callable_signature(code)
                        } else {
                            (Vec::new(), None)
                        };
                        let item_idx = items.len();
                        items.push(SourceAstItem {
                            kind: actual_kind,
                            name,
                            owner_name: active_impl_owner.clone(),
                            parent_type: None,
                            line_no,
                            doc_lines: current_doc.clone(),
                            fields: Vec::new(),
                            params,
                            return_type,
                            satisfied_requirements: reqs,
                            invariants: Vec::new(),
                            states: Vec::new(),
                        });
                        if actual_kind.is_callable() {
                            active_fn_idx = Some(item_idx);
                            if !code.contains('{') && !code.ends_with(';') {
                                pending_sig_fn_idx = Some(item_idx);
                                pending_sig_text = code.to_string();
                            }
                        }
                        current_doc.clear();
                    }
                } else {
                    if let Some(f_idx) = active_fn_idx {
                        if let Some(inv) = extract_assertion_invariant(code) {
                            if let Some(f_item) = items.get_mut(f_idx) {
                                f_item.invariants.push(inv);
                            }
                        }
                    }
                    cfg_test_pending = false;
                    current_doc.clear();
                }
            } else {
                // Inside function or method body (`brace_depth >= 2`)
                if let Some((kind, name)) = classify_rust_item_line_ast(code) {
                    if !cfg_test_pending {
                        let reqs = extract_realises_ids_from_doc_lines(&current_doc);
                        let (params, return_type) = if kind.is_callable() {
                            parse_rust_callable_signature(code)
                        } else {
                            (Vec::new(), None)
                        };
                        items.push(SourceAstItem {
                            kind,
                            name,
                            owner_name: active_impl_owner.clone(),
                            parent_type: None,
                            line_no,
                            doc_lines: current_doc.clone(),
                            fields: Vec::new(),
                            params,
                            return_type,
                            satisfied_requirements: reqs,
                            invariants: Vec::new(),
                            states: Vec::new(),
                        });
                    }
                    cfg_test_pending = false;
                    current_doc.clear();
                } else {
                    if let Some(f_idx) = active_fn_idx {
                        if let Some(inv) = extract_assertion_invariant(code) {
                            if let Some(f_item) = items.get_mut(f_idx) {
                                f_item.invariants.push(inv);
                            }
                        }
                    }
                    cfg_test_pending = false;
                    current_doc.clear();
                }
            }
        }

        update_ast_delimiter_depths(code, &mut brace_depth, &mut paren_depth);
        if brace_depth == 0 {
            active_struct_idx = None;
            active_enum_idx = None;
            if active_impl_owner.is_some() {
                active_impl_owner = None;
                pending_impl_header = None;
            }
            active_fn_idx = None;
        } else if brace_depth == 1 && active_impl_owner.is_some() {
            active_fn_idx = None;
        }
    }

    Ok(items)
}

/// Parses a Dart source string into a declaration-ordered vector of `SourceAstItem` records.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and FEAT-42 AC-01 Dart AST extraction (`parse_dart_source_ast`).
/// Parses Dart `class` (including `StatefulWidget`, `StatelessWidget`, and `State<T>`), `enum`,
/// `mixin`, `extension`, `typedef`, constructors, methods, properties, callback ports, and
/// `/// Realises: [...]` requirement bindings without regular expressions.
///
/// ## 2. Invariants & Mathematical Properties
/// - Validates delimiter balance via `validate_source_syntax_for_language(..., SourceLanguage::Dart)` first.
/// - Tracks `(brace_depth, paren_depth)` so named parameter braces `({super.key})` inside `(...)`
///   never corrupt class body brace depth.
/// - Classifies callback and stream fields (`VoidCallback`, `ValueChanged<T>`, `Function`, `Stream<T>`,
///   or `on<Event>` handlers) with `is_callback_or_port = true`.
///
/// ## 3. Preconditions
/// - `source: &str` contains UTF-8 Dart source code.
/// - `file_label: &str` is the file path used for syntax error diagnostics.
///
/// ## 4. Postconditions
/// - Returns `Ok(Vec<SourceAstItem>)` containing all extracted Dart classes, members, and functions,
///   or `Err(String)` if syntax validation fails.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(L) where L is the line count of `source`.
/// - Space Complexity: O(I) where I is the number of extracted Dart AST items and fields.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Rejects malformed Dart syntax with `file:line:col` diagnostics.
/// - State: Isolated per-invocation state machine.
/// - Action: Non-panicking slice and character iteration.
/// - Resource: Bounded heap memory proportional to source declarations.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-PARSE-DART-AST]
pub fn parse_dart_source_ast(
    source: &str,
    file_label: &str,
) -> Result<Vec<SourceAstItem>, CoreError> {
    validate_source_syntax_for_language(source, file_label, SourceLanguage::Dart)?;

    let raw_lines: Vec<&str> = source.lines().collect();
    let mut items: Vec<SourceAstItem> = Vec::new();
    let mut current_doc: Vec<String> = Vec::new();
    let mut dart_lex_state = RustDartLexState {
        language: SourceLanguage::Dart,
        ..RustDartLexState::default()
    };
    let mut scoped_state = ScopedItemScannerState::default();
    let mut brace_depth: usize = 0;
    let mut paren_depth: usize = 0;
    let mut active_class_idx: Option<usize> = None;
    let mut active_class_name: Option<String> = None;
    let mut active_fn_idx: Option<usize> = None;

    for (idx, &raw_line) in raw_lines.iter().enumerate() {
        let line_no = idx.saturating_add(1);
        let trimmed = raw_line.trim();
        let started_in_normal = dart_lex_state.block_comment_depth == 0
            && dart_lex_state.active_string.is_none()
            && dart_lex_state.raw_hash_count.is_none();

        if started_in_normal && trimmed.starts_with("///") {
            current_doc.push(trimmed.to_string());
            continue;
        }

        let lex_sanitized = scan_line_with_unified_lex_state(trimmed, &mut dart_lex_state);
        if lex_sanitized.trim().is_empty()
            || (!started_in_normal
                && (dart_lex_state.block_comment_depth > 0
                    || dart_lex_state.active_string.is_some()))
        {
            continue;
        }

        let code = strip_line_comment_preserving_strings(trimmed).trim();
        let stripped_code = strip_dart_annotations_tracking_state(
            code,
            &mut scoped_state.dart_annotation_paren_depth,
        );
        if stripped_code.trim().is_empty() {
            continue;
        }
        let code = stripped_code.trim();

        if paren_depth == 0 {
            if brace_depth == 0 {
                if let Some((kind, name, parent_type)) = classify_dart_top_level_ast(code) {
                    let reqs = extract_realises_ids_from_doc_lines(&current_doc);
                    let (params, return_type) = if kind.is_callable() {
                        let sig_code = collect_dart_callable_signature(&raw_lines, idx);
                        parse_dart_callable_signature(&sig_code, &name)
                    } else {
                        (Vec::new(), None)
                    };
                    let item_idx = items.len();
                    items.push(SourceAstItem {
                        kind,
                        name: name.clone(),
                        owner_name: None,
                        parent_type,
                        line_no,
                        doc_lines: current_doc.clone(),
                        fields: Vec::new(),
                        params,
                        return_type,
                        satisfied_requirements: reqs,
                        invariants: Vec::new(),
                        states: Vec::new(),
                    });
                    if kind == SourceItemKind::Class
                        || kind == SourceItemKind::Mixin
                        || kind == SourceItemKind::Extension
                        || kind == SourceItemKind::Enum
                    {
                        active_class_idx = Some(item_idx);
                        active_class_name = Some(name);
                    } else if kind.is_callable() {
                        active_fn_idx = Some(item_idx);
                    }
                    current_doc.clear();
                } else {
                    current_doc.clear();
                }
            } else if brace_depth == 1 {
                if let Some(ref cls_name) = active_class_name.clone() {
                    let is_enum = active_class_idx
                        .and_then(|i| items.get(i))
                        .map(|it| it.kind == SourceItemKind::Enum)
                        .unwrap_or(false);
                    if is_enum {
                        if let Some(v) = parse_enum_variant_line(code) {
                            if let Some(c_idx) = active_class_idx {
                                if let Some(c_item) = items.get_mut(c_idx) {
                                    c_item.states.push(v);
                                }
                            }
                        }
                        current_doc.clear();
                    } else if let Some((member_kind, member_name)) =
                        classify_dart_member_ast(code, cls_name)
                    {
                        let reqs = extract_realises_ids_from_doc_lines(&current_doc);
                        let sig_code = collect_dart_callable_signature(&raw_lines, idx);
                        let (params, return_type) =
                            parse_dart_callable_signature(&sig_code, &member_name);
                        let m_idx = items.len();
                        items.push(SourceAstItem {
                            kind: member_kind,
                            name: member_name,
                            owner_name: Some(cls_name.clone()),
                            parent_type: None,
                            line_no,
                            doc_lines: current_doc.clone(),
                            fields: Vec::new(),
                            params,
                            return_type,
                            satisfied_requirements: reqs,
                            invariants: Vec::new(),
                            states: Vec::new(),
                        });
                        active_fn_idx = Some(m_idx);
                        current_doc.clear();
                    } else if let Some(field) = parse_dart_field_line(code, &current_doc) {
                        if let Some(c_idx) = active_class_idx {
                            if let Some(c_item) = items.get_mut(c_idx) {
                                c_item.fields.push(field);
                            }
                        }
                        current_doc.clear();
                    } else {
                        current_doc.clear();
                    }
                } else {
                    if let Some(f_idx) = active_fn_idx {
                        if let Some(inv) = extract_assertion_invariant(code) {
                            if let Some(f_item) = items.get_mut(f_idx) {
                                f_item.invariants.push(inv);
                            }
                        }
                    }
                    current_doc.clear();
                }
            } else {
                if let Some(f_idx) = active_fn_idx {
                    if let Some(inv) = extract_assertion_invariant(code) {
                        if let Some(f_item) = items.get_mut(f_idx) {
                            f_item.invariants.push(inv);
                        }
                    }
                }
                current_doc.clear();
            }
        }

        let prev_brace_depth = brace_depth;
        update_ast_delimiter_depths(code, &mut brace_depth, &mut paren_depth);
        if (prev_brace_depth > 0 && brace_depth == 0)
            || (brace_depth == 0
                && paren_depth == 0
                && (code.ends_with(';') || code.ends_with('}')))
        {
            active_class_idx = None;
            active_class_name = None;
            active_fn_idx = None;
        } else if brace_depth == 1
            && paren_depth == 0
            && active_class_name.is_some()
            && (prev_brace_depth >= 2 || code.ends_with(';') || code.ends_with('}'))
        {
            active_fn_idx = None;
        }
    }

    resolve_owner_field_params(&mut items);
    Ok(items)
}

/// Lowers a slice of `SourceAstItem` declarations into a canonical SysML v2 `PackageDef`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and FEAT-42 Section 3.1 M2 metamodel mapping rules (`lower_source_ast_to_package`).
/// Maps classes and structs to `PartDef`, fields to `AttributeDef`, callbacks/streams to `PortDef`,
/// methods and functions to `ActionDef`, `/// Realises: [REQ-ID]` tags to `satisfied_requirements`
/// (`satisfy requirement <ReqId>;`), and runtime assertions to `ConstraintDef`.
///
/// ## 2. Invariants & Mathematical Properties
/// - 100% M2 Metamodel Purity: Emits only closed-grammar SysML v2 AST structures (`PackageDef`,
///   `PartDef`, `AttributeDef`, `PortDef`, `ActionDef`, `StateDef`, `ConstraintDef`, `RequirementDef`).
/// - Links Dart `State<WidgetName>` classes to their owning `WidgetName` `PartDef` so widget parts
///   contain both widget properties/callbacks and state actions/transitions.
///
/// ## 3. Preconditions
/// - `items: &[SourceAstItem]` contains extracted declarations from one or more source files.
/// - `package_name: &str` is the logical package or directory name.
///
/// ## 4. Postconditions
/// - Returns a deterministic `PackageDef` ready for `to_sysml` serialization and SHA-256 digest computation.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(I * P) where I is `items.len()` and P is the count of `PartDef`s in the package.
/// - Space Complexity: O(I) to hold the lowered `PackageDef` tree.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Sanitizes `package_name` and source type names into valid SysML v2 identifiers.
/// - State: Pure functional transformation producing a fresh `PackageDef`.
/// - Action: Deduplicates requirement definitions and satisfaction bindings deterministically.
/// - Resource: Bounded heap allocations proportional to `items.len()`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-LOWER-AST-TO-PACKAGE]
pub fn lower_source_ast_to_package(items: &[SourceAstItem], package_name: &str) -> PackageDef {
    let clean_pkg_name = sanitize_sysml_identifier(package_name, "SourceCodeModel");
    let mut pkg = PackageDef {
        name: clean_pkg_name.clone(),
        ..Default::default()
    };
    let mut registered_reqs: BTreeSet<String> = BTreeSet::new();
    let mut state_to_widget: std::collections::BTreeMap<String, String> =
        std::collections::BTreeMap::new();
    for item in items {
        if item.kind == SourceItemKind::Class {
            if let Some(ref parent) = item.parent_type {
                if let Some(widget_name) = extract_state_generic_target(parent) {
                    let state_clean = sanitize_sysml_identifier(&item.name, "ComponentPart");
                    let widget_clean = sanitize_sysml_identifier(&widget_name, "ComponentPart");
                    state_to_widget.insert(state_clean, widget_clean);
                }
            }
        }
    }

    // Pass 1: Create PartDef and StateDef nodes for structs, classes, traits, mixins, extensions, and enums
    for item in items {
        for req_id in &item.satisfied_requirements {
            if registered_reqs.insert(req_id.clone()) {
                pkg.requirement_defs.push(RequirementDef {
                    name: sanitize_sysml_identifier(req_id, "REQ_ITEM"),
                    req_id: req_id.clone(),
                    text: format!("Requirement {} realized by {}", req_id, item.name),
                    ..Default::default()
                });
            }
        }

        match item.kind {
            SourceItemKind::Struct
            | SourceItemKind::Class
            | SourceItemKind::Trait
            | SourceItemKind::Mixin
            | SourceItemKind::Extension
            | SourceItemKind::Typedef => {
                let part_name = sanitize_sysml_identifier(&item.name, "ComponentPart");
                let mut attributes = Vec::new();
                let mut ports = Vec::new();
                for field in &item.fields {
                    let clean_field_name = sanitize_sysml_identifier(&field.name, "field_attr");
                    let clean_type = normalize_type_for_sysml(&field.type_name);
                    if field.is_callback_or_port {
                        ports.push(PortDef {
                            name: clean_field_name,
                            type_name: clean_type,
                            direction: "inout".to_string(),
                            port_category: "DataPort".to_string(),
                            doc: field.doc.clone(),
                            ..Default::default()
                        });
                    } else {
                        attributes.push(AttributeDef {
                            name: clean_field_name,
                            type_name: clean_type,
                            default_value: None,
                            doc: field.doc.clone(),
                        });
                    }
                }

                // If this Dart class is `State<OwnerWidget>`, also attach its attributes/ports and a StateDef to `OwnerWidget`
                if let Some(widget_clean) = state_to_widget.get(&part_name).cloned() {
                    if !pkg.part_defs.iter().any(|p| p.name == widget_clean) {
                        pkg.part_defs.push(PartDef {
                            name: widget_clean.clone(),
                            is_def: true,
                            ..Default::default()
                        });
                    }
                    if let Some(owner_part) =
                        pkg.part_defs.iter_mut().find(|p| p.name == widget_clean)
                    {
                        owner_part.attributes.extend(attributes.clone());
                        owner_part.ports.extend(ports.clone());
                        for req in &item.satisfied_requirements {
                            if !owner_part.satisfied_requirements.contains(req) {
                                owner_part.satisfied_requirements.push(req.clone());
                            }
                        }
                        owner_part.states.push(StateDef {
                            name: part_name.clone(),
                            doc: summary_from_doc_lines(&item.doc_lines),
                            ..Default::default()
                        });
                    }
                }

                if let Some(existing) = pkg.part_defs.iter_mut().find(|p| p.name == part_name) {
                    if existing.doc.is_none() {
                        existing.doc = summary_from_doc_lines(&item.doc_lines);
                    }
                    existing.attributes.extend(attributes);
                    existing.ports.extend(ports);
                    for req in &item.satisfied_requirements {
                        if !existing.satisfied_requirements.contains(req) {
                            existing.satisfied_requirements.push(req.clone());
                        }
                    }
                } else {
                    pkg.part_defs.push(PartDef {
                        name: part_name,
                        doc: summary_from_doc_lines(&item.doc_lines),
                        is_def: true,
                        attributes,
                        ports,
                        satisfied_requirements: item.satisfied_requirements.clone(),
                        ..Default::default()
                    });
                }
            }
            SourceItemKind::Enum => {
                let enum_name = sanitize_sysml_identifier(&item.name, "StateEnum");
                let sub_states: Vec<StateDef> = item
                    .states
                    .iter()
                    .map(|v| StateDef {
                        name: sanitize_sysml_identifier(v, "StateVariant"),
                        ..Default::default()
                    })
                    .collect();
                pkg.state_defs.push(StateDef {
                    name: enum_name.clone(),
                    doc: summary_from_doc_lines(&item.doc_lines),
                    sub_states: sub_states.clone(),
                    ..Default::default()
                });
                if !pkg.part_defs.iter().any(|p| p.name == enum_name) {
                    pkg.part_defs.push(PartDef {
                        name: enum_name,
                        doc: summary_from_doc_lines(&item.doc_lines),
                        is_def: true,
                        states: sub_states,
                        satisfied_requirements: item.satisfied_requirements.clone(),
                        ..Default::default()
                    });
                }
            }
            _ => {}
        }
    }

    // Pass 2: Lower functions, methods, and constructors into ActionDef and ConstraintDef
    for item in items {
        let resolved_item = resolve_callable_param_types(item, items);
        let Some((in_sig, out_sig, _)) = extract_method_signature(&resolved_item) else {
            continue;
        };
        let act_name = sanitize_sysml_identifier(&item.name, "action_op");
        let in_params: Vec<AttributeDef> = in_sig
            .into_iter()
            .map(|(name, type_name)| AttributeDef {
                name,
                type_name,
                default_value: None,
                doc: None,
            })
            .collect();
        let out_params: Vec<AttributeDef> = out_sig
            .into_iter()
            .map(|(name, type_name)| AttributeDef {
                name,
                type_name,
                default_value: None,
                doc: None,
            })
            .collect();

        let action_def = ActionDef {
            name: act_name.clone(),
            doc: summary_from_doc_lines(&item.doc_lines),
            in_params,
            out_params,
            parameters: Vec::new(),
            performer: item.owner_name.clone(),
            steps: Vec::new(),
            is_def: true,
        };

        if let Some(ref owner_raw) = item.owner_name {
            let owner_clean = sanitize_sysml_identifier(owner_raw, "ComponentPart");
            if !pkg.part_defs.iter().any(|p| p.name == owner_clean) {
                pkg.part_defs.push(PartDef {
                    name: owner_clean.clone(),
                    is_def: true,
                    ..Default::default()
                });
            }
            if let Some(owner_part) = pkg.part_defs.iter_mut().find(|p| p.name == owner_clean) {
                owner_part.actions.push(action_def.clone());
                for req in &item.satisfied_requirements {
                    if !owner_part.satisfied_requirements.contains(req) {
                        owner_part.satisfied_requirements.push(req.clone());
                    }
                }
                for (inv_idx, inv_expr) in item.invariants.iter().enumerate() {
                    owner_part.constraints.push(ConstraintDef {
                        name: format!("invariant_{}_{}", act_name, inv_idx.saturating_add(1)),
                        expression: inv_expr.clone(),
                        is_assertion: true,
                        ..Default::default()
                    });
                }
            }
            if let Some(widget_clean) = state_to_widget.get(&owner_clean).cloned() {
                if !pkg.part_defs.iter().any(|p| p.name == widget_clean) {
                    pkg.part_defs.push(PartDef {
                        name: widget_clean.clone(),
                        is_def: true,
                        ..Default::default()
                    });
                }
                if let Some(widget_part) = pkg.part_defs.iter_mut().find(|p| p.name == widget_clean)
                {
                    if !widget_part
                        .actions
                        .iter()
                        .any(|a| a.name == action_def.name)
                    {
                        widget_part.actions.push(action_def);
                    }
                    for req in &item.satisfied_requirements {
                        if !widget_part.satisfied_requirements.contains(req) {
                            widget_part.satisfied_requirements.push(req.clone());
                        }
                    }
                    for (inv_idx, inv_expr) in item.invariants.iter().enumerate() {
                        let inv_name =
                            format!("invariant_{}_{}", act_name, inv_idx.saturating_add(1));
                        if !widget_part.constraints.iter().any(|c| c.name == inv_name) {
                            widget_part.constraints.push(ConstraintDef {
                                name: inv_name,
                                expression: inv_expr.clone(),
                                is_assertion: true,
                                ..Default::default()
                            });
                        }
                    }
                }
            }
        } else {
            pkg.action_defs.push(action_def);
            for req in &item.satisfied_requirements {
                if !pkg.satisfied_requirements.contains(req) {
                    pkg.satisfied_requirements.push(req.clone());
                }
            }
            for (inv_idx, inv_expr) in item.invariants.iter().enumerate() {
                pkg.constraint_defs.push(ConstraintDef {
                    name: format!("invariant_{}_{}", act_name, inv_idx.saturating_add(1)),
                    expression: inv_expr.clone(),
                    is_assertion: true,
                    ..Default::default()
                });
            }
        }
    }

    // Ensure at least one PartDef exists when top-level functions were ingested without a struct/class
    if pkg.part_defs.is_empty() && !pkg.action_defs.is_empty() {
        pkg.part_defs.push(PartDef {
            name: format!("{}Module", clean_pkg_name),
            is_def: true,
            actions: pkg.action_defs.clone(),
            satisfied_requirements: pkg.satisfied_requirements.clone(),
            ..Default::default()
        });
    }

    pkg
}

/// Resolves a deterministic SysML v2 package identifier for an ingested source directory.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, and Issue #472 namespace isolation rules
/// (`resolve_source_package_name`). Prevents multi-crate Rust (`<component>/src`) and
/// Flutter (`<component>/lib`) source trees from colliding under unscoped `package src`
/// or `package lib` root namespaces when no explicit `--package` override is provided.
///
/// ## 2. Invariants & Mathematical Properties
/// - When `explicit_package` contains a non-empty trimmed identifier, returns its sanitized
///   SysML v2 representation directly.
/// - When `explicit_package` is `None` (or empty) and the leaf directory of `source_dir` is
///   `"src"` or `"lib"` with a valid parent directory name `<Parent>`, synthesizes
///   `<Parent>_<src|lib>_Realization`.
/// - Otherwise sanitizes the leaf directory name with fallback `"SourceCodeModel"`.
///
/// ## 3. Preconditions
/// - `source_dir: &Path` is the filesystem path being ingested.
/// - `explicit_package: Option<&str>` is an optional caller-supplied package override.
///
/// ## 4. Postconditions
/// - Returns a non-empty, syntactically valid SysML v2 package identifier string.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(P)$ where $P$ is the character length of the path leaf and parent components.
/// - Space Complexity: $O(P)$ heap allocation for the resolved identifier string.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Gracefully handles root paths or relative paths lacking a parent component by
///   falling back to the sanitized leaf name or `"SourceCodeModel"`.
/// - State: Pure function with zero filesystem or global state mutation.
/// - Action: Never panics on arbitrary OS path structures.
/// - Resource: Bounded single `String` allocation.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-RESOLVE-SOURCE-PACKAGE-NAME], [REQ-ISSUE-472-PACKAGE-QUALIFICATION]
pub fn resolve_source_package_name(source_dir: &Path, explicit_package: Option<&str>) -> String {
    if let Some(pkg) = explicit_package.map(str::trim).filter(|s| !s.is_empty()) {
        return sanitize_sysml_identifier(pkg, "SourceCodeModel");
    }
    let leaf = source_dir
        .file_name()
        .and_then(|n| n.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("SourceCodeModel");
    if matches!(leaf, "src" | "lib") {
        if let Some(parent_name) = source_dir
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .filter(|s| !s.is_empty() && *s != "." && *s != "..")
        {
            return sanitize_sysml_identifier(
                &format!("{}_{}_Realization", parent_name, leaf),
                "SourceCodeModel",
            );
        }
    }
    sanitize_sysml_identifier(leaf, "SourceCodeModel")
}

/// Recursively scans `source_dir` for `.rs` and/or `.dart` files and lowers them into a canonical `PackageDef`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and FEAT-42 Section 2.1 source directory ingestion contract (`ingest_source_directory`).
/// Walks `source_dir` in deterministic lexicographical order, parses every matching source file
/// without regular expressions, and produces a unified SysML v2 `PackageDef`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Lexicographical file sorting guarantees bit-for-bit identical `PackageDef` outputs across platforms.
/// - Any syntax error in any discovered source file halts ingestion immediately with `Err(String)`.
/// - Raw `"src"` or `"lib"` package identifiers are automatically parent-qualified via [`resolve_source_package_name`].
///
/// ## 3. Preconditions
/// - `source_dir: &Path` is an existing file or directory path on disk.
/// - `language: SourceLanguage` specifies `Auto`, `Rust`, or `Dart`.
/// - `package_name: &str` provides the root SysML v2 package identifier or raw leaf fallback.
///
/// ## 4. Postconditions
/// - Returns `Ok(PackageDef)` when at least one eligible source file is parsed cleanly, or `Err(String)`
///   if the path is inaccessible, contains zero source files, or fails syntax verification.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N log N) where N is the total byte size and file count under `source_dir`.
/// - Space Complexity: O(N) for accumulated `SourceAstItem` and `PackageDef` nodes.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Non-existent directories or empty file sets fail closed with descriptive error messages.
/// - State: Accumulates into local vectors without mutating filesystem state.
/// - Action: Short-circuits on the first file read or syntax error via `?`.
/// - Resource: Releases per-file string buffers immediately after AST extraction.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-INGEST-SOURCE-DIR]
pub fn ingest_source_directory(
    source_dir: &Path,
    language: SourceLanguage,
    package_name: &str,
) -> Result<PackageDef, CoreError> {
    if !source_dir.exists() {
        return Err(CoreError::io(format!(
            "Source path '{}' does not exist",
            source_dir.display()
        )));
    }

    let files = discover_source_files(source_dir, language)?;
    if files.is_empty() {
        return Err(CoreError::validation(format!(
            "No eligible '{}' source files (.rs / .dart) found in '{}'",
            language.label(),
            source_dir.display()
        )));
    }

    let mut all_items: Vec<SourceAstItem> = Vec::new();
    for file_path in &files {
        let content = fs::read_to_string(file_path).map_err(|e| {
            CoreError::io(format!(
                "Failed to read source file '{}': {}",
                file_path.display(),
                e
            ))
        })?;
        let file_label = file_path.display().to_string();
        let file_lang = match language {
            SourceLanguage::Auto => {
                SourceLanguage::detect_from_extension(file_path).unwrap_or(SourceLanguage::Rust)
            }
            explicit => explicit,
        };
        let mut parsed = match file_lang {
            SourceLanguage::Rust | SourceLanguage::Auto => {
                parse_rust_source_ast(&content, &file_label)?
            }
            SourceLanguage::Dart => parse_dart_source_ast(&content, &file_label)?,
        };
        all_items.append(&mut parsed);
    }

    let effective_pkg_name = if matches!(package_name.trim(), "" | "src" | "lib") {
        resolve_source_package_name(source_dir, None)
    } else {
        resolve_source_package_name(source_dir, Some(package_name))
    };

    Ok(lower_source_ast_to_package(&all_items, &effective_pkg_name))
}

/// Discovers eligible `.rs` and/or `.dart` source files under `root` in deterministic sorted order.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A workspace traversal rules (`discover_source_files`).
/// Excludes build caches (`target`, `build`, `.dart_tool`), VCS metadata (`.git`), and generated
/// Dart artifacts (`.g.dart`, `.freezed.dart`) while preserving deterministic path ordering.
///
/// ## 2. Invariants & Mathematical Properties
/// - Returned `Vec<PathBuf>` is strictly sorted in lexicographical order.
/// - When `root` is a single file, validates that its extension matches `language`.
///
/// ## 3. Preconditions
/// - `root: &Path` is an existing file or directory path.
/// - `language: SourceLanguage` filters for `.rs`, `.dart`, or both (`Auto`).
///
/// ## 4. Postconditions
/// - Returns `Ok(Vec<PathBuf>)` listing all discovered source files, or `Err(String)` on walk error.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(F log F) where F is the number of filesystem entries under `root`.
/// - Space Complexity: O(F) for the collected `PathBuf` vector.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Converts `WalkDir` I/O errors into explicit `Err(String)` diagnostics (fail-closed).
/// - State: Stateless directory traversal.
/// - Action: Skips non-matching extensions and generated Dart files deterministically.
/// - Resource: Bounded by workspace file count.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-DISCOVER-SOURCE-FILES]
pub fn discover_source_files(
    root: &Path,
    language: SourceLanguage,
) -> Result<Vec<PathBuf>, CoreError> {
    if root.is_file() {
        return Ok(vec![root.to_path_buf()]);
    }

    let mut files = Vec::new();
    for entry_res in WalkDir::new(root)
        .follow_links(false)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|e| {
            if e.depth() == 0 {
                return true;
            }
            let dname = e.file_name().to_string_lossy();
            if dname.starts_with('.') || dname.starts_with('#') {
                return false;
            }
            if e.file_type().is_dir() {
                !matches!(
                    dname.as_ref(),
                    "target"
                        | "node_modules"
                        | "build"
                        | "Pods"
                        | "ephemeral"
                        | "generated"
                        | "vendor"
                        | "dist"
                        | "tests"
                        | "benches"
                        | "examples"
                )
            } else {
                true
            }
        })
    {
        let entry = entry_res.map_err(|e| {
            CoreError::io(format!(
                "Directory walk error under '{}': {}",
                root.display(),
                e
            ))
        })?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let fname = entry.file_name().to_string_lossy();
        if fname == "generated_plugin_registrant.dart"
            || fname.ends_with(".g.dart")
            || fname.ends_with(".freezed.dart")
            || fname.ends_with(".gr.dart")
            || fname.ends_with(".mocks.dart")
        {
            continue;
        }
        if let Some(detected) = SourceLanguage::detect_from_extension(path) {
            if language == SourceLanguage::Auto || language == detected {
                files.push(path.to_path_buf());
            }
        }
    }

    files.sort();
    Ok(files)
}

/// Extracts all bracketed requirement IDs from `Realises: [...]` lines in `doc_lines`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and FEAT-42 Section 3.1 requirement traceability extraction (`extract_realises_ids_from_doc_lines`).
/// Parses single or comma-separated `[REQ-ID]` tokens on `Realises:` docstring lines into formal
/// `satisfied_requirements` bindings.
///
/// ## 2. Invariants & Mathematical Properties
/// - Preserves exact requirement ID characters (including hyphens and zero-padded numeric segments).
/// - Deduplicates IDs while preserving declaration order.
///
/// ## 3. Preconditions
/// - `doc_lines: &[String]` contains trimmed `///` or `//!` docstring lines.
///
/// ## 4. Postconditions
/// - Returns `Vec<String>` of all non-empty bracketed requirement identifiers following `"Realises:"`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(D) where D is the total byte length of `doc_lines`.
/// - Space Complexity: O(R) where R is the count of extracted requirement IDs.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Ignores malformed or unclosed brackets safely via `split_once`.
/// - State: Pure function with local `Vec<String>` accumulator.
/// - Action: Never panics; uses `.get(..)` and `split_once`.
/// - Resource: Bounded by docstring line length.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-EXTRACT-REALISES-IDS]
pub fn extract_realises_ids_from_doc_lines(doc_lines: &[String]) -> Vec<String> {
    let mut ids = Vec::new();
    for line in doc_lines {
        if let Some((_, after_realises)) = line.split_once("Realises:") {
            let mut cursor = after_realises;
            while let Some((_, after_open)) = cursor.split_once('[') {
                if let Some((inside, rest)) = after_open.split_once(']') {
                    for part in inside.split(',') {
                        let clean = part.trim();
                        if !clean.is_empty() && !ids.iter().any(|existing| existing == clean) {
                            ids.push(clean.to_string());
                        }
                    }
                    cursor = rest;
                } else {
                    break;
                }
            }
        }
    }
    ids
}

/// Extracts the leading summary sentence from `doc_lines`, if present.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A documentation preservation rules (`summary_from_doc_lines`).
/// Retains human-authored summary descriptions when lowering source declarations or re-synthesizing
/// incomplete docstrings.
///
/// ## 2. Invariants & Mathematical Properties
/// - Strips leading `///` / `//!` markers and returns `Some(String)` only if the first non-empty
///   line is not a section header (`## `) and not a `Realises:` tag.
///
/// ## 3. Preconditions
/// - `doc_lines: &[String]` is a slice of contiguous docstring lines.
///
/// ## 4. Postconditions
/// - Returns `Some(summary)` for a valid leading summary line, or `None` if absent.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(L) where L is the length of the first non-empty doc line.
/// - Space Complexity: O(L) for the returned summary `String`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Handles empty `doc_lines` slices cleanly by returning `None`.
/// - State: Pure function.
/// - Action: Rejects placeholder `"Member documentation."` lines so placeholders are never preserved.
/// - Resource: At most one small `String` allocation.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-SUMMARY-FROM-DOC-LINES]
pub fn summary_from_doc_lines(doc_lines: &[String]) -> Option<String> {
    for line in doc_lines {
        let stripped = line
            .trim_start_matches("///")
            .trim_start_matches("//!")
            .trim();
        if stripped.is_empty() {
            continue;
        }
        let lower = stripped.to_ascii_lowercase();
        let member_placeholder = ["member ", "documentation"].concat();
        if stripped.starts_with('#')
            || stripped.starts_with("Realises:")
            || stripped.starts_with("///")
            || lower.starts_with(&member_placeholder)
        {
            return None;
        }
        return Some(stripped.to_string());
    }
    None
}

/// Extracts the top-level header prefix (up to and excluding the opening `{`) of a Rust `impl` block.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for `extract_rust_impl_header_prefix` (`REQ-0605` `AC-02`).
///
/// ## 2. Invariants & Mathematical Properties
/// - Accepts `impl` and `unsafe impl` headers with arbitrary generic `<...>` and `Fn(...) -> ...` bounds.
/// - Scans for the first opening `{` at `angle_depth == 0 && paren_depth == 0` while ignoring `>` inside `->`.
///
/// ## 3. Preconditions
/// - `code: &str` is a comment-stripped Rust `impl` header line or accumulated multi-line `impl` header.
///
/// ## 4. Postconditions
/// - Returns `Some(prefix)` containing the trimmed `impl` header before `{`, or `None` if `code` is not a complete `impl` header.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) where N is `code.len()`.
/// - Space Complexity: O(1) zero-copy slice borrow.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Non-panicking `.get(..)` slicing and character iteration.
/// - State: Stateless pure function.
/// - Action: Returns `None` when `code` does not start with `impl` / `unsafe impl` or lacks a top-level `{`.
/// - Resource: Zero heap allocations.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0605], [REQ-FEAT-42-PARSE-RUST-IMPL-HEADER]
pub fn extract_rust_impl_header_prefix(code: &str) -> Option<&str> {
    let trimmed = code.trim();
    let after_unsafe = trimmed
        .strip_prefix("unsafe ")
        .unwrap_or(trimmed)
        .trim_start();
    let after_impl = after_unsafe.strip_prefix("impl")?;
    if !after_impl.is_empty()
        && !after_impl.starts_with('<')
        && !after_impl.starts_with(char::is_whitespace)
    {
        return None;
    }

    let mut angle_depth: usize = 0;
    let mut paren_depth: usize = 0;
    let mut prev_ch: Option<char> = None;
    for (i, ch) in trimmed.char_indices() {
        match ch {
            '<' => angle_depth = angle_depth.saturating_add(1),
            '>' if prev_ch != Some('-') => angle_depth = angle_depth.saturating_sub(1),
            '(' => paren_depth = paren_depth.saturating_add(1),
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '{' if angle_depth == 0 && paren_depth == 0 => {
                return trimmed.get(..i).map(str::trim);
            }
            _ => {}
        }
        prev_ch = Some(ch);
    }
    None
}

/// Parses a Rust `impl` block header line and returns the target type identifier.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and FEAT-42 AC-02 `impl` block association (`parse_rust_impl_header`).
/// Binds methods declared inside `impl ComponentBuffer` or `impl Trait for ComponentBuffer` to the
/// owning `ComponentBuffer` classifier.
///
/// ## 2. Invariants & Mathematical Properties
/// - Strips optional generic parameter lists `impl<...>` and trait prefixes `<Trait> for ` before
///   extracting the target type identifier.
///
/// ## 3. Preconditions
/// - `code: &str` is a comment-stripped Rust line at `brace_depth == 0`.
///
/// ## 4. Postconditions
/// - Returns `Some(target_type_name)` when `code` begins an `impl` block, or `None` otherwise.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) where N is `code.len()`.
/// - Space Complexity: O(I) where I is the length of the extracted target identifier.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Non-panicking prefix stripping and character scanning.
/// - State: Stateless lexical helper.
/// - Action: Returns `None` if `code` is not an `impl` header or has no `{`.
/// - Resource: Single identifier `String` allocation.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0605], [REQ-FEAT-42-PARSE-RUST-IMPL-HEADER]
pub fn parse_rust_impl_header(code: &str) -> Option<String> {
    let before_brace = extract_rust_impl_header_prefix(code)?;
    let after_unsafe = before_brace
        .strip_prefix("unsafe ")
        .unwrap_or(before_brace)
        .trim_start();
    let rest = if let Some(r) = after_unsafe
        .strip_prefix("impl")
        .map(str::trim_start)
        .and_then(|s| s.strip_prefix('<'))
    {
        let mut depth: usize = 1;
        let mut end_idx: Option<usize> = None;
        let mut prev_ch: Option<char> = None;
        for (i, ch) in r.char_indices() {
            if ch == '<' {
                depth = depth.saturating_add(1);
            } else if ch == '>' && prev_ch != Some('-') {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    end_idx = Some(i.saturating_add(1));
                    break;
                }
            }
            prev_ch = Some(ch);
        }
        r.get(end_idx?..).unwrap_or("").trim_start()
    } else {
        after_unsafe.strip_prefix("impl ")?.trim_start()
    };

    let without_where = rest
        .split_once(" where ")
        .map(|(before, _)| before.trim())
        .unwrap_or_else(|| rest.strip_suffix(" where").map(str::trim).unwrap_or(rest));

    let mut angle_depth: usize = 0;
    let mut paren_depth: usize = 0;
    let mut prev_ch: Option<char> = None;
    let mut top_level_for_end: Option<usize> = None;
    for (i, ch) in without_where.char_indices() {
        if angle_depth == 0
            && paren_depth == 0
            && without_where
                .get(i..)
                .map(|s| s.starts_with(" for "))
                .unwrap_or(false)
        {
            top_level_for_end = Some(i.saturating_add(" for ".len()));
            break;
        }
        match ch {
            '<' => angle_depth = angle_depth.saturating_add(1),
            '>' if prev_ch != Some('-') => angle_depth = angle_depth.saturating_sub(1),
            '(' => paren_depth = paren_depth.saturating_add(1),
            ')' => paren_depth = paren_depth.saturating_sub(1),
            _ => {}
        }
        prev_ch = Some(ch);
    }

    let target_part = match top_level_for_end {
        Some(idx) => without_where.get(idx..).unwrap_or("").trim(),
        None => without_where,
    };
    let target_clean = target_part
        .trim_start_matches('&')
        .trim_start_matches("mut ")
        .trim();
    extract_leading_ast_ident(target_clean)
}

/// Classifies a comment-stripped Rust line as a `Struct`, `Enum`, `Trait`, or `Function` declaration.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and Check 36 item classification parity (`classify_rust_item_line_ast`).
/// Ensures `source_ast` and `compile-sysml --inject-safety-docstrings` recognize the exact same
/// Rust declarations enforced by Check 36 (`audit_rust_source_docstrings`).
///
/// ## 2. Invariants & Mathematical Properties
/// - Strips visibility (`pub`, `pub(crate)`, `pub(super)`, `pub(in ...)`) and qualifiers (`const`, `async`, `unsafe`, `extern`).
/// - Matches `struct `, `enum `, `trait `, and `fn ` declarations followed by a valid identifier.
///
/// ## 3. Preconditions
/// - `code: &str` is a trimmed, comment-stripped Rust source line outside `mod tests`.
///
/// ## 4. Postconditions
/// - Returns `Some((SourceItemKind, identifier))` for eligible Rust items, or `None` otherwise.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) where N is `code.len()`.
/// - Space Complexity: O(I) where I is the length of the item identifier.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Safe prefix stripping via `strip_prefix` and `split_once`.
/// - State: Pure function.
/// - Action: Never panics on arbitrary line content.
/// - Resource: Single `String` allocation on match.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-CLASSIFY-RUST-ITEM-AST]
pub fn classify_rust_item_line_ast(code: &str) -> Option<(SourceItemKind, String)> {
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
        let name = extract_leading_ast_ident(rest.trim_start())?;
        return Some((SourceItemKind::Struct, name));
    }
    if let Some(rest) = after_vis.strip_prefix("enum ") {
        let name = extract_leading_ast_ident(rest.trim_start())?;
        return Some((SourceItemKind::Enum, name));
    }
    let after_trait_qual = after_vis
        .strip_prefix("unsafe ")
        .map_or(after_vis, str::trim_start);
    if let Some(rest) = after_trait_qual.strip_prefix("trait ") {
        let name = extract_leading_ast_ident(rest.trim_start())?;
        return Some((SourceItemKind::Trait, name));
    }

    let mut after_qual = after_vis;
    loop {
        if let Some(rest) = after_qual.strip_prefix("const ") {
            after_qual = rest.trim_start();
        } else if let Some(rest) = after_qual.strip_prefix("async ") {
            after_qual = rest.trim_start();
        } else if let Some(rest) = after_qual.strip_prefix("unsafe ") {
            after_qual = rest.trim_start();
        } else if let Some(rest) = after_qual.strip_prefix("extern") {
            if !rest.starts_with(' ') && !rest.starts_with('"') {
                break;
            }
            let trimmed = rest.trim_start();
            if let Some(after_quote) = trimmed.strip_prefix('"') {
                if let Some((_, after_abi)) = after_quote.split_once('"') {
                    after_qual = after_abi.trim_start();
                } else {
                    break;
                }
            } else {
                after_qual = trimmed;
            }
        } else {
            break;
        }
    }

    if let Some(rest) = after_qual.strip_prefix("fn ") {
        let name = extract_leading_ast_ident(rest.trim_start())?;
        return Some((SourceItemKind::Function, name));
    }

    None
}

/// Parses a single field line inside a Rust `struct` body into a `SourceField`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and FEAT-42 AC-02 struct field extraction (`parse_rust_struct_field_line`).
/// Extracts `name: Type` members from Rust `struct` definitions for lowering into `AttributeDef` or `PortDef`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Strips optional `pub`, `pub(crate)`, `pub(super)` visibility prefixes before splitting on `':'`.
/// - Classifies fields with `Callback`, `Fn`, `Sender`, `Receiver`, or `Stream` types as `is_callback_or_port = true`.
///
/// ## 3. Preconditions
/// - `code: &str` is a trimmed line at `brace_depth == 1` inside an active Rust `struct`.
/// - `doc_lines: &[String]` holds any `///` lines preceding the field.
///
/// ## 4. Postconditions
/// - Returns `Some(SourceField)` for valid `name: Type` lines, or `None` for closing braces/attributes.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) where N is `code.len()`.
/// - Space Complexity: O(N) for owned `name` and `type_name` strings.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Ignores tuple struct lines or brace lines lacking `':'`.
/// - State: Pure function.
/// - Action: Uses `split_once(':')` without byte indexing.
/// - Resource: Bounded by single line length.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-PARSE-RUST-STRUCT-FIELD]
fn parse_rust_struct_field_line(code: &str, doc_lines: &[String]) -> Option<SourceField> {
    if code.starts_with('}') || code.starts_with('#') || code.contains("fn ") {
        return None;
    }
    let after_vis = if let Some(r) = code.strip_prefix("pub(crate) ") {
        r.trim_start()
    } else if let Some(r) = code.strip_prefix("pub(super) ") {
        r.trim_start()
    } else if let Some(r) = code.strip_prefix("pub ") {
        r.trim_start()
    } else {
        code
    };
    let (raw_name, raw_type_rest) = after_vis.split_once(':')?;
    let name = extract_leading_ast_ident(raw_name.trim())?;
    let type_name = raw_type_rest
        .trim()
        .trim_end_matches(',')
        .trim()
        .to_string();
    if type_name.is_empty() {
        return None;
    }
    let is_port = is_callback_or_port_field(&name, &type_name);
    Some(SourceField {
        name,
        type_name,
        is_callback_or_port: is_port,
        doc: summary_from_doc_lines(doc_lines),
    })
}

/// Parses formal parameters and return type from a Rust `fn` signature string.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and FEAT-42 AC-02/AC-03 callable signature extraction (`parse_rust_callable_signature`).
/// Extracts `(param_name: ParamType)` inputs and `-> ReturnType` outputs for `ActionDef` lowering
/// and Section 3/4 docstring synthesis.
///
/// ## 2. Invariants & Mathematical Properties
/// - Excludes receiver tokens (`self`, `&self`, `&mut self`, `mut self`) from formal `SourceParam` list.
/// - Splits top-level parameters on `','` only when nested angle/paren/bracket depth is zero.
///
/// ## 3. Preconditions
/// - `sig: &str` contains a Rust `fn` header up to `{` or `;`.
///
/// ## 4. Postconditions
/// - Returns `(Vec<SourceParam>, Option<String>)` representing formal inputs and optional return type.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) where N is `sig.len()`.
/// - Space Complexity: O(P) where P is the number of parameters.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Gracefully handles functions without parameters or return types.
/// - State: Pure lexical parser.
/// - Action: Uses `.get(..)` and depth counters to split generic parameter types safely.
/// - Resource: Bounded by signature string length.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-PARSE-RUST-CALLABLE-SIG]
pub fn parse_rust_callable_signature(sig: &str) -> (Vec<SourceParam>, Option<String>) {
    let mut params = Vec::new();
    let Some((before_open, after_open)) = sig.split_once('(') else {
        return (params, None);
    };
    let _fn_prefix = before_open;

    let mut depth: usize = 1;
    let mut close_idx: Option<usize> = None;
    for (i, ch) in after_open.char_indices() {
        if ch == '(' {
            depth = depth.saturating_add(1);
        } else if ch == ')' {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                close_idx = Some(i);
                break;
            }
        }
    }

    let Some(c_idx) = close_idx else {
        return (params, None);
    };

    let param_section = after_open.get(..c_idx).unwrap_or("");
    for raw_param in split_top_level_comma(param_section) {
        let trimmed = raw_param.trim();
        if trimmed.is_empty()
            || trimmed == "self"
            || trimmed == "&self"
            || trimmed == "&mut self"
            || trimmed == "mut self"
            || trimmed.starts_with("&'") && trimmed.ends_with("self")
        {
            continue;
        }
        if let Some((pname_raw, ptype_raw)) = trimmed.split_once(':') {
            let clean_pname = pname_raw
                .trim()
                .strip_prefix("mut ")
                .unwrap_or(pname_raw.trim())
                .trim_start_matches('&')
                .trim();
            let pname =
                extract_leading_ast_ident(clean_pname).unwrap_or_else(|| "param".to_string());
            let ptype = ptype_raw.trim().to_string();
            if !ptype.is_empty() {
                params.push(SourceParam {
                    name: pname,
                    type_name: ptype,
                });
            }
        }
    }

    let after_close = after_open.get(c_idx.saturating_add(1)..).unwrap_or("");
    let return_type = if let Some((_, after_arrow)) = after_close.split_once("->") {
        let before_where = after_arrow
            .split_once(" where ")
            .map(|(b, _)| b)
            .unwrap_or(after_arrow);
        let before_brace = before_where
            .split_once('{')
            .map(|(b, _)| b)
            .unwrap_or(before_where);
        let clean_ret = before_brace.trim().trim_end_matches(';').trim();
        if clean_ret.is_empty() {
            None
        } else {
            Some(clean_ret.to_string())
        }
    } else {
        None
    };

    (params, return_type)
}

/// Classifies a top-level (`brace_depth == 0`) Dart line and extracts its optional parent class (`extends <Parent>`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and FEAT-42 AC-01 Dart class and function classification (`classify_dart_top_level_ast`).
/// Identifies Dart `class`, `enum`, `mixin`, `extension`, `typedef`, and top-level functions while
/// capturing `extends <SuperType>` (such as `StatefulWidget` or `State<MyWidget>`).
///
/// ## 2. Invariants & Mathematical Properties
/// - Matches the exact top-level Dart construct taxonomy enforced by Check 36 (`classify_dart_top_level_item`).
/// - Extracts optional superclass text following `" extends "` up to `" with "`, `" implements "`, or `'{'`.
///
/// ## 3. Preconditions
/// - `code: &str` is a comment-stripped Dart line at `brace_depth == 0` and `paren_depth == 0`.
///
/// ## 4. Postconditions
/// - Returns `Some((SourceItemKind, name, parent_type))` for top-level Dart declarations, or `None`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) where N is `code.len()`.
/// - Space Complexity: O(I) for the extracted identifier and optional parent type strings.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Ignores `import`, `export`, `part`, `library`, and top-level variable lines.
/// - State: Pure lexical classifier.
/// - Action: Non-panicking prefix stripping.
/// - Resource: Bounded string allocations.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-CLASSIFY-DART-TOP-AST]
pub fn classify_dart_top_level_ast(code: &str) -> Option<(SourceItemKind, String, Option<String>)> {
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
        let name = extract_leading_ast_ident(after_mixin_class.trim_start())?;
        let parent = extract_dart_extends_type(after_mixin_class);
        return Some((SourceItemKind::Class, name, parent));
    }
    if let Some(after_class) = rest.strip_prefix("class ") {
        let name = extract_leading_ast_ident(after_class.trim_start())?;
        let parent = extract_dart_extends_type(after_class);
        return Some((SourceItemKind::Class, name, parent));
    }
    if let Some(after_enum) = rest.strip_prefix("enum ") {
        let name = extract_leading_ast_ident(after_enum.trim_start())?;
        return Some((SourceItemKind::Enum, name, None));
    }
    if let Some(after_mixin) = rest.strip_prefix("mixin ") {
        let name = extract_leading_ast_ident(after_mixin.trim_start())?;
        return Some((SourceItemKind::Mixin, name, None));
    }
    if let Some(after_ext_type) = rest.strip_prefix("extension type ") {
        let name = extract_leading_ast_ident(after_ext_type.trim_start())?;
        return Some((SourceItemKind::Extension, name, None));
    }
    if let Some(after_ext) = rest.strip_prefix("extension ") {
        let name = extract_leading_ast_ident(after_ext.trim_start())
            .unwrap_or_else(|| "anonymous_extension".to_string());
        return Some((SourceItemKind::Extension, name, None));
    }
    if let Some(after_typedef) = rest.strip_prefix("typedef ") {
        let name = extract_leading_ast_ident(after_typedef.trim_start())?;
        return Some((SourceItemKind::Typedef, name, None));
    }

    if let Some(getter_name) = extract_dart_getter_ident(code) {
        return Some((SourceItemKind::Function, getter_name, None));
    }

    if let Some(fn_name) = extract_dart_callable_ident(code) {
        return Some((SourceItemKind::Function, fn_name, None));
    }

    None
}

/// Classifies a class-body (`brace_depth == 1`) Dart line as a `Constructor` or `Method`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and Check 36 Dart member parity (`classify_dart_member_ast`).
/// Identifies `const MyApp({super.key});`, named constructors, factory constructors, getters, and class
/// methods while excluding field declarations.
///
/// ## 2. Invariants & Mathematical Properties
/// - Matches the exact member classification rules of `classify_dart_member_item` in Check 36.
///
/// ## 3. Preconditions
/// - `code: &str` is a comment-stripped Dart line at `brace_depth == 1` and `paren_depth == 0`.
/// - `class_name: &str` is the enclosing Dart classifier name.
///
/// ## 4. Postconditions
/// - Returns `Some((SourceItemKind::Constructor | SourceItemKind::Method, member_name))` or `None`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) where N is `code.len()`.
/// - Space Complexity: O(I) where I is the length of the member name.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Non-panicking prefix and delimiter checks.
/// - State: Pure function.
/// - Action: Distinguishes `const ClassName(` constructors from `const` fields.
/// - Resource: Single `String` allocation on match.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-CLASSIFY-DART-MEMBER-AST]
pub fn classify_dart_member_ast(code: &str, class_name: &str) -> Option<(SourceItemKind, String)> {
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
            if let Some(named) = extract_leading_ast_ident(rest) {
                return Some((SourceItemKind::Constructor, format!("{}.{}", cls, named)));
            }
        }
        let cname = extract_leading_ast_ident(trimmed_after)?;
        return Some((SourceItemKind::Constructor, cname));
    }

    let after_const = if let Some(rest) = code.strip_prefix("const ") {
        rest.trim_start()
    } else {
        code
    };

    if let Some(after_cls) = after_const.strip_prefix(class_name) {
        let trimmed_after = after_cls.trim_start();
        if trimmed_after.starts_with('(') {
            return Some((SourceItemKind::Constructor, class_name.to_string()));
        }
        if let Some(after_dot) = trimmed_after.strip_prefix('.') {
            if let Some(named_ctor) = extract_leading_ast_ident(after_dot) {
                return Some((
                    SourceItemKind::Constructor,
                    format!("{}.{}", class_name, named_ctor),
                ));
            }
        }
    }

    if code.starts_with("const ") {
        return None;
    }

    if let Some(getter_name) = extract_dart_getter_ident(code) {
        return Some((SourceItemKind::Method, getter_name));
    }

    let after_mods = code.strip_prefix("static ").unwrap_or(code).trim_start();
    if let Some(method_name) = extract_dart_callable_ident(after_mods) {
        if !matches!(
            method_name.as_str(),
            "if" | "for" | "while" | "switch" | "return" | "assert" | "super"
        ) {
            return Some((SourceItemKind::Method, method_name));
        }
    }

    None
}

/// Parses a Dart class field declaration line at `brace_depth == 1` into a `SourceField`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and FEAT-42 AC-01 Dart property and callback extraction (`parse_dart_field_line`).
/// Extracts `final <Type> <name>;` and `<Type> <name>;` properties and classifies event callbacks
/// (`VoidCallback`, `ValueChanged<T>`, `Function`, `Stream<T>`, `on<Event>`) as `PortDef` candidates.
///
/// ## 2. Invariants & Mathematical Properties
/// - Strips `static`, `late`, `final`, `const`, and `covariant` field modifiers before extracting
///   `<Type> <name>`.
/// - Sets `is_callback_or_port = is_callback_or_port_field(&name, &type_name)`.
///
/// ## 3. Preconditions
/// - `code: &str` is a comment-stripped Dart line at `brace_depth == 1` that was not classified as a method/constructor.
/// - `doc_lines: &[String]` holds any `///` lines preceding the field.
///
/// ## 4. Postconditions
/// - Returns `Some(SourceField)` when `code` declares a Dart field ending with `;`, or `None`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) where N is `code.len()`.
/// - Space Complexity: O(N) for the extracted field name and type strings.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Safely ignores closing braces, control statements, and getters/setters.
/// - State: Pure function.
/// - Action: Non-panicking character and token splitting.
/// - Resource: Bounded by single line length.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-PARSE-DART-FIELD]
fn parse_dart_field_line(code: &str, doc_lines: &[String]) -> Option<SourceField> {
    if !code.ends_with(';') || code.starts_with('}') || code.starts_with("return ") {
        return None;
    }
    let without_semi = code.trim_end_matches(';').trim();
    let (before_eq, rhs_opt) = without_semi
        .split_once('=')
        .map(|(b, rhs)| (b.trim(), Some(rhs.trim())))
        .unwrap_or((without_semi, None));
    if before_eq.contains('(') && !before_eq.contains("Function(") {
        return None;
    }
    let rest = strip_dart_field_modifiers(before_eq);
    let mut rev_chars = rest.chars().rev();
    let mut name_rev = String::new();
    for ch in &mut rev_chars {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            name_rev.push(ch);
        } else if ch.is_whitespace() {
            break;
        } else {
            return None;
        }
    }
    if name_rev.is_empty() {
        return None;
    }
    let name: String = name_rev.chars().rev().collect();
    let type_part: String = rev_chars.collect::<String>().chars().rev().collect();
    let clean_type = if type_part.trim().is_empty() || type_part.trim() == "var" {
        infer_dart_literal_type(rhs_opt)
    } else {
        type_part.trim().to_string()
    };
    let is_port = is_callback_or_port_field(&name, &clean_type);
    Some(SourceField {
        name,
        type_name: clean_type,
        is_callback_or_port: is_port,
        doc: summary_from_doc_lines(doc_lines),
    })
}

/// Strips leading Dart field modifiers (`static`, `late`, `final`, `const`, `covariant`) from a declaration prefix.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for `strip_dart_field_modifiers`.
/// Isolates the `<Type> <fieldName>` or `var <fieldName>` segment of a Dart class field declaration.
///
/// ## 2. Invariants & Mathematical Properties
/// - Strips at most 4 leading modifier keywords in declaration order without regex or panicking slices.
///
/// ## 3. Preconditions
/// - `rest: &str` is the pre-`=` segment of a candidate Dart field line.
///
/// ## 4. Postconditions
/// - Returns the trimmed slice following all leading field modifiers.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) bounded 4-pass prefix check.
/// - Space Complexity: O(1) zero heap allocation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Accepts any UTF-8 slice and returns a subslice.
/// - State: Pure stateless helper.
/// - Action: Zero panics or unwraps.
/// - Resource: Zero dynamic memory.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-PARSE-DART-FIELD]
fn strip_dart_field_modifiers(mut rest: &str) -> &str {
    for _ in 0..4 {
        if let Some(r) = rest.strip_prefix("static ") {
            rest = r.trim_start();
        } else if let Some(r) = rest.strip_prefix("late ") {
            rest = r.trim_start();
        } else if let Some(r) = rest.strip_prefix("final ") {
            rest = r.trim_start();
        } else if let Some(r) = rest.strip_prefix("const ") {
            rest = r.trim_start();
        } else if let Some(r) = rest.strip_prefix("covariant ") {
            rest = r.trim_start();
        } else {
            break;
        }
    }
    rest
}

/// Infers a canonical Dart primitive type name (`"bool"`, `"int"`, `"double"`, or `"String"`) from an optional literal initializer.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for `infer_dart_literal_type`.
/// Preserves numeric and boolean type fidelity for untyped `final`, `const`, and `var` Dart field declarations and default parameter values (`#518`).
///
/// ## 2. Invariants & Mathematical Properties
/// - Maps `"true"` / `"false"` to `"bool"`.
/// - Maps signed/unsigned decimal integer literals to `"int"` and floating-point literals with a decimal point to `"double"`.
/// - Falls back to `"String"` for absent or non-primitive initializer expressions.
///
/// ## 3. Preconditions
/// - `rhs_opt: Option<&str>` is the optional trimmed right-hand side of `=` in a field or parameter declaration.
///
/// ## 4. Postconditions
/// - Returns a non-empty owned Dart type string ready for `normalize_type_for_sysml`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(L) where L is the byte length of `rhs_opt`.
/// - Space Complexity: O(1) bounded string allocation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Handles `None` or arbitrary expression syntax by returning `"String"`.
/// - State: Pure function.
/// - Action: Zero panics, unwraps, or regex operations.
/// - Resource: Single short `String` allocation.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-PARSE-DART-FIELD]
fn infer_dart_literal_type(rhs_opt: Option<&str>) -> String {
    let Some(raw) = rhs_opt else {
        return "String".to_string();
    };
    let trimmed = raw.trim();
    if trimmed == "true" || trimmed == "false" {
        return "bool".to_string();
    }
    let num_part = trimmed
        .strip_prefix('-')
        .or_else(|| trimmed.strip_prefix('+'))
        .unwrap_or(trimmed);
    if !num_part.is_empty()
        && num_part.chars().all(|c| c.is_ascii_digit() || c == '_')
        && num_part.chars().any(|c| c.is_ascii_digit())
    {
        return "int".to_string();
    }
    if let Some((int_part, frac_part)) = num_part.split_once('.') {
        if !int_part.is_empty()
            && !frac_part.is_empty()
            && int_part.chars().all(|c| c.is_ascii_digit() || c == '_')
            && frac_part.chars().all(|c| c.is_ascii_digit() || c == '_')
        {
            return "double".to_string();
        }
    }
    "String".to_string()
}

/// Collects a potentially multi-line Dart callable signature starting at `start_idx` until its parameter list closes.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for `collect_dart_callable_signature`.
/// Ensures multi-line Dart constructors, methods, and top-level functions preserve all formal parameters across line breaks.
///
/// ## 2. Invariants & Mathematical Properties
/// - Strips line comments (`strip_line_comment_preserving_strings`) on every joined line and leading Dart annotations (`strip_leading_dart_annotations`) on the opening line.
/// - Tracks balanced parenthesis depth via `update_ast_delimiter_depths` and terminates as soon as `'('` has been observed and `paren_depth == 0` (or after the first line when `'('` is absent, such as getters).
///
/// ## 3. Preconditions
/// - `raw_lines: &[&str]` is the full slice of lines in a Dart source file.
/// - `start_idx: usize` is the 0-based line index of a Dart callable declaration.
///
/// ## 4. Postconditions
/// - Returns a single whitespace-joined `String` containing the complete callable signature through its closing `')'`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(W) where W is the character count across at most 32 continuation lines.
/// - Space Complexity: O(W) for the joined signature `String`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Uses `.get(start_idx..end_idx)` with bounded lookahead (max 32 lines).
/// - State: Pure lexical aggregator with local delimiter depth state.
/// - Action: Zero panics, unwraps, or expectations.
/// - Resource: Bounded heap allocation proportional to signature length.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-COLLECT-DART-CALLABLE-SIG]
pub fn collect_dart_callable_signature(raw_lines: &[&str], start_idx: usize) -> String {
    let mut combined = String::new();
    let mut brace_depth: usize = 0;
    let mut paren_depth: usize = 0;
    let mut saw_open_paren = false;
    let end_idx = start_idx.saturating_add(32).min(raw_lines.len());
    let slice = raw_lines.get(start_idx..end_idx).unwrap_or(&[]);

    for (offset, raw_line) in slice.iter().enumerate() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with("///") || trimmed.starts_with("//") {
            continue;
        }
        let without_comment = strip_line_comment_preserving_strings(trimmed).trim();
        let cleaned = if offset == 0 {
            strip_leading_dart_annotations(without_comment).trim()
        } else {
            without_comment
        };
        if cleaned.is_empty() {
            continue;
        }
        if !combined.is_empty() {
            combined.push(' ');
        }
        combined.push_str(cleaned);
        if cleaned.contains('(') {
            saw_open_paren = true;
        }
        update_ast_delimiter_depths(cleaned, &mut brace_depth, &mut paren_depth);
        if !saw_open_paren || paren_depth == 0 {
            break;
        }
    }

    combined
}

/// Parses formal parameters and return type from a Dart function, method, or constructor line.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and FEAT-42 AC-01/AC-03 Dart callable signature extraction (`parse_dart_callable_signature`).
/// Handles positional, optional (`[...]`), and named (`{required int x, super.key, this.title}`)
/// Dart parameters and extracts the return type preceding `callable_name`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Strips enclosing `{...}` and `[...]` parameter group delimiters before splitting on top-level commas.
/// - Maps `super.key` / `this.<field>` shorthand parameters to `SourceParam` entries with inferred or explicit types.
///
/// ## 3. Preconditions
/// - `code: &str` is a Dart callable declaration line containing `'('`.
/// - `callable_name: &str` is the identifier of the function, method, or constructor.
///
/// ## 4. Postconditions
/// - Returns `(Vec<SourceParam>, Option<String>)` with extracted formal parameters and return type.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) where N is `code.len()`.
/// - Space Complexity: O(P) where P is parameter count.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Handles unclosed parameter lists on multi-line constructors safely.
/// - State: Pure lexical parser.
/// - Action: Non-panicking `.get(..)` and character iteration.
/// - Resource: Bounded by declaration line length.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-PARSE-DART-CALLABLE-SIG]
pub fn parse_dart_callable_signature(
    code: &str,
    callable_name: &str,
) -> (Vec<SourceParam>, Option<String>) {
    let mut params = Vec::new();
    let Some((before_open, after_open)) = code.split_once('(') else {
        return (params, None);
    };
    let return_type = extract_dart_callable_return_type(before_open, callable_name);
    let param_section = slice_balanced_dart_param_section(after_open);
    for top_seg in split_top_level_comma(param_section.trim()) {
        let seg = top_seg.trim();
        if seg.is_empty() {
            continue;
        }
        if seg.starts_with('{') || seg.starts_with('[') {
            let inner = seg
                .trim_start_matches('{')
                .trim_end_matches('}')
                .trim_start_matches('[')
                .trim_end_matches(']')
                .trim();
            for inner_seg in split_top_level_comma(inner) {
                if let Some(parsed) = parse_single_dart_param(inner_seg) {
                    params.push(parsed);
                }
            }
        } else if let Some(parsed) = parse_single_dart_param(seg) {
            params.push(parsed);
        }
    }
    (params, return_type)
}

/// Extracts the declared return type preceding `callable_name` in a Dart signature prefix.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for `extract_dart_callable_return_type`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Strips `callable_name` and optional leading `static ` modifier; returns `None` for `const`/`factory` constructors or empty prefixes.
///
/// ## 3. Preconditions
/// - `before_open: &str` is the substring preceding `'('` in a Dart callable signature.
/// - `callable_name: &str` is the callable's identifier.
///
/// ## 4. Postconditions
/// - Returns `Some(return_type)` when an explicit return type is present, or `None`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) where N is `before_open.len()`.
/// - Space Complexity: O(R) for the extracted return type string.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Non-panicking `strip_suffix` and `strip_prefix`.
/// - State: Pure helper.
/// - Action: Zero panics or unwraps.
/// - Resource: Bounded single `String` allocation.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-PARSE-DART-CALLABLE-SIG]
fn extract_dart_callable_return_type(before_open: &str, callable_name: &str) -> Option<String> {
    let trimmed_before = before_open.trim();
    let prefix = trimmed_before.strip_suffix(callable_name)?;
    let clean_ret = prefix
        .trim()
        .strip_prefix("static ")
        .unwrap_or(prefix.trim())
        .trim();
    if clean_ret.is_empty() || clean_ret == "const" || clean_ret == "factory" {
        None
    } else {
        Some(clean_ret.to_string())
    }
}

/// Returns the balanced formal parameter substring inside `(` ... `)` from `after_open`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for `slice_balanced_dart_param_section`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Tracks nested parentheses and single/double-quoted string literals to locate the matching closing `')'`.
///
/// ## 3. Preconditions
/// - `after_open: &str` is the slice immediately following the opening `'('` of a Dart callable signature.
///
/// ## 4. Postconditions
/// - Returns the slice up to the matching `')'`, or `after_open` if unclosed.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) where N is `after_open.len()`.
/// - Space Complexity: O(1) zero heap allocation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Uses `.get(..c_idx)` exclusively.
/// - State: Local scalar counters and flags.
/// - Action: Zero panics or unwraps.
/// - Resource: Zero dynamic memory.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-PARSE-DART-CALLABLE-SIG]
fn slice_balanced_dart_param_section(after_open: &str) -> &str {
    let mut depth: usize = 1;
    let mut in_single = false;
    let mut in_double = false;
    let mut escaped = false;
    let mut close_idx: Option<usize> = None;
    for (i, ch) in after_open.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' && (in_single || in_double) {
            escaped = true;
            continue;
        }
        if ch == '\'' && !in_double {
            in_single = !in_single;
            continue;
        }
        if ch == '"' && !in_single {
            in_double = !in_double;
            continue;
        }
        if !in_single && !in_double {
            if ch == '(' {
                depth = depth.saturating_add(1);
            } else if ch == ')' {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    close_idx = Some(i);
                    break;
                }
            }
        }
    }
    match close_idx {
        Some(c_idx) => after_open.get(..c_idx).unwrap_or(""),
        None => after_open,
    }
}

/// Parses a single formal parameter token (including `this.<field>`, `super.<param>`, `<Type> this.<field>`, and `<Type> <name>`) into a `SourceParam`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for `parse_single_dart_param` (`#518`).
///
/// ## 2. Invariants & Mathematical Properties
/// - Strips parameter modifiers (`required`, `covariant`, `final`, `var`) and default value expressions (`= <rhs>`).
/// - Preserves explicit `<Type>` before `this.<field>` / `super.<param>`, or infers literal default types via `infer_dart_literal_type` without hardcoding `"Key"`.
///
/// ## 3. Preconditions
/// - `raw_param: &str` is a single comma-delimited parameter segment from a Dart callable signature.
///
/// ## 4. Postconditions
/// - Returns `Some(SourceParam)` for valid parameter tokens, or `None` when empty.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(P) where P is `raw_param.len()`.
/// - Space Complexity: O(P) for the parameter name and type strings.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Non-panicking prefix/delimiter splitting.
/// - State: Pure function.
/// - Action: Zero panics, unwraps, or regex operations.
/// - Resource: Bounded by single parameter token length.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-PARSE-DART-CALLABLE-SIG]
fn parse_single_dart_param(raw_param: &str) -> Option<SourceParam> {
    let mut p = raw_param
        .trim()
        .trim_start_matches('{')
        .trim_end_matches('}')
        .trim_start_matches('[')
        .trim_end_matches(']')
        .trim();
    if p.is_empty() {
        return None;
    }
    let (before_default, rhs_opt) = p
        .split_once('=')
        .map(|(b, r)| (b.trim(), Some(r.trim())))
        .unwrap_or((p, None));
    p = before_default;
    for prefix in ["required ", "covariant ", "final ", "var "] {
        if let Some(after) = p.strip_prefix(prefix) {
            p = after.trim();
        }
    }
    for marker in ["super.", "this."] {
        if let Some((type_prefix, after_marker)) = p.split_once(marker) {
            if type_prefix.is_empty() || type_prefix.ends_with(char::is_whitespace) {
                let ident = extract_leading_ast_ident(after_marker.trim())?;
                let explicit_type = type_prefix.trim();
                let type_name = if explicit_type.is_empty() {
                    infer_dart_literal_type(rhs_opt)
                } else {
                    explicit_type.to_string()
                };
                return Some(SourceParam {
                    name: ident,
                    type_name,
                });
            }
        }
    }
    let mut rev_chars = p.chars().rev();
    let mut pname_rev = String::new();
    for ch in &mut rev_chars {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            pname_rev.push(ch);
        } else {
            break;
        }
    }
    let pname: String = pname_rev.chars().rev().collect();
    if pname.is_empty() {
        return None;
    }
    let ptype_raw: String = rev_chars.collect::<String>().chars().rev().collect();
    let ptype = if ptype_raw.trim().is_empty() {
        infer_dart_literal_type(rhs_opt)
    } else {
        ptype_raw.trim().to_string()
    };
    Some(SourceParam {
        name: pname,
        type_name: ptype,
    })
}

/// Resolves `this.<field>` and `super.<param>` parameter types in-place across a parsed slice of `SourceAstItem` nodes.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for `resolve_owner_field_params` (`#518`).
///
/// ## 2. Invariants & Mathematical Properties
/// - Updates callable parameter types whose initial type is `"String"` when a matching field declaration exists on the owning class or its superclass chain in `items`.
///
/// ## 3. Preconditions
/// - `items: &mut [SourceAstItem]` contains the declarations extracted from a source translation unit.
///
/// ## 4. Postconditions
/// - Mutates `items` in-place so constructor and method parameters reflect declared class/superclass field types.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(I * P * C) where I is item count, P is parameter count, and C is class count.
/// - Space Complexity: O(I) for the read-only snapshot vector.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Preserves existing parameter types when no matching field exists.
/// - State: Bounded in-place slice mutation.
/// - Action: Zero panics or unwraps.
/// - Resource: Bounded snapshot clone.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-PARSE-DART-CALLABLE-SIG]
fn resolve_owner_field_params(items: &mut [SourceAstItem]) {
    let snapshot = items.to_vec();
    for item in items.iter_mut() {
        *item = resolve_callable_param_types(item, &snapshot);
    }
}

/// Returns a copy of `item` with callable parameter types resolved against owning class and superclass fields in `all_items`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for `resolve_callable_param_types` (`#518`).
///
/// ## 2. Invariants & Mathematical Properties
/// - For callable items with `owner_name: Some(owner)`, any parameter whose `type_name == "String"` is resolved via `lookup_owner_field_type(owner, &param.name, all_items)`.
///
/// ## 3. Preconditions
/// - `item: &SourceAstItem` is a candidate AST item; `all_items: &[SourceAstItem]` is the full package AST item slice.
///
/// ## 4. Postconditions
/// - Returns a `SourceAstItem` with resolved `SourceParam::type_name` entries.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(P * C) where P is `item.params.len()` and C is `all_items.len()`.
/// - Space Complexity: O(P) for the cloned item.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Leaves non-callable or ownerless items unchanged.
/// - State: Pure function.
/// - Action: Zero panics or unwraps.
/// - Resource: Single `SourceAstItem` clone.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-LOWER-AST-TO-PACKAGE]
fn resolve_callable_param_types(
    item: &SourceAstItem,
    all_items: &[SourceAstItem],
) -> SourceAstItem {
    let mut resolved = item.clone();
    if !resolved.kind.is_callable() {
        return resolved;
    }
    let Some(ref owner) = resolved.owner_name else {
        return resolved;
    };
    for param in &mut resolved.params {
        if param.type_name == "String" {
            if let Some(field_ty) = lookup_owner_field_type(owner, &param.name, all_items) {
                param.type_name = field_ty;
            }
        }
    }
    resolved
}

/// Looks up the declared `type_name` of `field_name` on `owner_name` or its superclass chain (up to 4 hops) in `all_items`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for `lookup_owner_field_type` (`#518`).
///
/// ## 2. Invariants & Mathematical Properties
/// - Traverses at most 4 inheritance hops via `parent_type` to prevent unbounded loops on cyclic class hierarchies.
///
/// ## 3. Preconditions
/// - `owner_name: &str` and `field_name: &str` are non-empty identifiers; `all_items: &[SourceAstItem]` contains class/struct items.
///
/// ## 4. Postconditions
/// - Returns `Some(type_name)` when a matching `SourceField` is found on the class or a superclass, or `None`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(C + F) bounded by 4 superclass hops over `all_items`.
/// - Space Complexity: O(1) auxiliary memory.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Returns `None` when the owner or superclass is external to `all_items`.
/// - State: Pure lookup.
/// - Action: Zero panics or unwraps.
/// - Resource: At most one `String` clone on match.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-PARSE-DART-CALLABLE-SIG]
fn lookup_owner_field_type(
    owner_name: &str,
    field_name: &str,
    all_items: &[SourceAstItem],
) -> Option<String> {
    let mut current_owner = owner_name.trim();
    for _ in 0..4 {
        let cls = all_items
            .iter()
            .find(|i| !i.kind.is_callable() && i.name == current_owner)?;
        if let Some(field) = cls.fields.iter().find(|f| f.name == field_name) {
            return Some(field.type_name.clone());
        }
        let parent_raw = cls.parent_type.as_deref()?.trim();
        let parent_base = parent_raw
            .split_once('<')
            .map(|(b, _)| b.trim())
            .unwrap_or(parent_raw);
        if parent_base.is_empty() || parent_base == current_owner {
            return None;
        }
        current_owner = parent_base;
    }
    None
}

/// Determines whether a struct or class field represents a callback handler or stream port (`PortDef`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and FEAT-42 Section 3.1 `PortDef` mapping (`is_callback_or_port_field`).
/// Identifies asynchronous notification sinks, UI callbacks (`VoidCallback`, `ValueChanged<T>`,
/// `onPressed`, `onChanged`), and streams (`Stream<T>`, `Sender<T>`, `Receiver<T>`).
///
/// ## 2. Invariants & Mathematical Properties
/// - Returns `true` if `type_name` contains callback/channel/stream type markers or if `field_name`
///   matches the `on<Uppercase>` event handler naming pattern.
///
/// ## 3. Preconditions
/// - `field_name: &str` and `type_name: &str` are non-empty trimmed identifiers/types.
///
/// ## 4. Postconditions
/// - Returns `true` when the field must be lowered to `PortDef`, or `false` for `AttributeDef`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) where N is `field_name.len() + type_name.len()`.
/// - Space Complexity: O(1) zero heap allocation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Non-panicking character and substring checks.
/// - State: Pure predicate.
/// - Action: Deterministic boolean evaluation.
/// - Resource: Zero allocations.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-IS-CALLBACK-OR-PORT]
fn is_callback_or_port_field(field_name: &str, type_name: &str) -> bool {
    let has_on_prefix = if let Some(after_on) = field_name.strip_prefix("on") {
        after_on
            .chars()
            .next()
            .map(|c| c.is_ascii_uppercase() || c == '_')
            .unwrap_or(false)
    } else {
        false
    };

    has_on_prefix
        || type_name.contains("Callback")
        || type_name.contains("Function")
        || type_name.contains("ValueChanged")
        || type_name.contains("ValueSetter")
        || type_name.contains("ValueGetter")
        || type_name.contains("Stream")
        || type_name.contains("EventSink")
        || type_name.contains("Sender")
        || type_name.contains("Receiver")
        || type_name.starts_with("Fn(")
        || type_name.starts_with("FnMut(")
        || type_name.starts_with("FnOnce(")
}

/// Extracts an enum variant identifier from a single line inside a Rust or Dart `enum` body.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A state-space extraction (`parse_enum_variant_line`).
/// Populates `StateDef::sub_states` from enum variant declarations.
///
/// ## 2. Invariants & Mathematical Properties
/// - Ignores closing braces, attributes, annotations, and method lines inside Dart enhanced enums.
/// - Returns the leading identifier when followed by `,`, `{`, `(`, `=`, or end-of-line.
///
/// ## 3. Preconditions
/// - `code: &str` is a comment-stripped line at `brace_depth == 1` inside an `enum`.
///
/// ## 4. Postconditions
/// - Returns `Some(variant_name)` for enum variant lines, or `None` for non-variant lines.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) where N is `code.len()`.
/// - Space Complexity: O(V) where V is the variant identifier length.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Safe character iteration via `extract_leading_ast_ident`.
/// - State: Pure helper.
/// - Action: Never panics.
/// - Resource: At most one small `String` allocation.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-PARSE-ENUM-VARIANT]
fn parse_enum_variant_line(code: &str) -> Option<String> {
    if code.starts_with('}')
        || code.starts_with('#')
        || code.starts_with('@')
        || code.starts_with("const ")
        || code.starts_with("final ")
        || code.starts_with("fn ")
        || code.starts_with("pub ")
    {
        return None;
    }
    extract_leading_ast_ident(code)
}

/// Extracts runtime assertion expressions (`assert!(...)`, `debug_assert!(...)`, `assert(...)`) from a code line.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and FEAT-42 Section 3.1 runtime assertion lowering (`extract_assertion_invariant`).
/// Lowers executable `assert!` and `assert(...)` statements into formal SysML v2 `ConstraintDef` assertions.
///
/// ## 2. Invariants & Mathematical Properties
/// - Extracts the first balanced argument expression inside `assert!(...)`, `debug_assert!(...)`, or `assert(...)`.
///
/// ## 3. Preconditions
/// - `code: &str` is a comment-stripped line inside a function or method body.
///
/// ## 4. Postconditions
/// - Returns `Some(expr)` containing the assertion condition, or `None` if `code` is not an assertion.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) where N is `code.len()`.
/// - Space Complexity: O(N) for the extracted expression string.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Uses `strip_prefix` and `split_top_level_comma` without panicking range indexing.
/// - State: Pure function.
/// - Action: Sanitizes string literals or semicolons so the expression is valid in `ConstraintDef`.
/// - Resource: Single `String` allocation on match.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-EXTRACT-ASSERTION-INVARIANT]
fn extract_assertion_invariant(code: &str) -> Option<String> {
    let inner = if let Some(rest) = code.strip_prefix("assert!(") {
        rest.rsplit_once(')').map(|(i, _)| i)
    } else if let Some(rest) = code.strip_prefix("debug_assert!(") {
        rest.rsplit_once(')').map(|(i, _)| i)
    } else if let Some(rest) = code.strip_prefix("assert(") {
        rest.rsplit_once(')').map(|(i, _)| i)
    } else {
        None
    }?;

    let first_arg = split_top_level_comma(inner).into_iter().next()?.trim();
    if first_arg.is_empty() {
        return None;
    }
    Some(first_arg.replace('"', "'"))
}

/// Extracts the superclass expression following `extends ` in a Dart class header.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and FEAT-42 AC-01 Dart widget hierarchy resolution (`extract_dart_extends_type`).
/// Captures `StatefulWidget`, `StatelessWidget`, or `State<MyWidget>` from Dart `class` headers.
///
/// ## 2. Invariants & Mathematical Properties
/// - Slices the substring after `" extends "` up to `" with "`, `" implements "`, or `'{'`.
///
/// ## 3. Preconditions
/// - `after_class_kw: &str` is the remainder of a Dart `class` line after `"class "`.
///
/// ## 4. Postconditions
/// - Returns `Some(super_type)` if `" extends "` is present, or `None`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) where N is `after_class_kw.len()`.
/// - Space Complexity: O(P) where P is the superclass token length.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Uses `split_once` with zero byte-index slicing.
/// - State: Pure helper.
/// - Action: Never panics.
/// - Resource: At most one `String` allocation.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-EXTRACT-DART-EXTENDS]
fn extract_dart_extends_type(after_class_kw: &str) -> Option<String> {
    let (_, after_extends) = after_class_kw.split_once(" extends ")?;
    let before_brace = after_extends
        .split_once('{')
        .map(|(b, _)| b)
        .unwrap_or(after_extends);
    let before_with = before_brace
        .split_once(" with ")
        .map(|(b, _)| b)
        .unwrap_or(before_brace);
    let before_impl = before_with
        .split_once(" implements ")
        .map(|(b, _)| b)
        .unwrap_or(before_with);
    let clean = before_impl.trim();
    if clean.is_empty() {
        None
    } else {
        Some(clean.to_string())
    }
}

/// Extracts `WidgetName` from a Dart `State<WidgetName>` superclass string.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and FEAT-42 AC-01 `StatefulWidget` state-class binding (`extract_state_generic_target`).
/// Links `_MyWidgetState extends State<MyWidget>` back to the `MyWidget` `PartDef`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Returns `Some(WidgetName)` iff `parent_type` starts with `"State<"` and ends with `'>'`.
///
/// ## 3. Preconditions
/// - `parent_type: &str` is the superclass string extracted by `extract_dart_extends_type`.
///
/// ## 4. Postconditions
/// - Returns `Some(sanitized_widget_name)` or `None`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) where N is `parent_type.len()`.
/// - Space Complexity: O(N) for the extracted widget identifier.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Uses `strip_prefix` and `strip_suffix` safely.
/// - State: Pure function.
/// - Action: Returns `None` on non-`State<...>` parent types.
/// - Resource: Single `String` allocation on match.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-EXTRACT-STATE-GENERIC]
fn extract_state_generic_target(parent_type: &str) -> Option<String> {
    let after_state = parent_type.trim().strip_prefix("State<")?;
    let inside = after_state.strip_suffix('>')?.trim();
    extract_leading_ast_ident(inside)
}

/// Normalizes a Rust or Dart type annotation into a canonical SysML v2 single-token type identifier.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and FEAT-42 Section 3.1 type lowering rules (`normalize_type_for_sysml`).
/// Ensures all lowered `AttributeDef`, `PortDef`, and `ActionDef` type names are valid SysML v2
/// identifiers accepted by both `SysmlParser` and `SemanticValidator`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Strips references (`&`, `&mut`), lifetimes (`'a`), pointers, slices, and nullable `?` suffixes.
/// - Maps generic containers (`Vec<T>`, `Option<T>`, `Result<T, E>`, `ValueChanged<T>`) to their
///   outer or inner canonical identifier so serialized SysML v2 (`attribute x : Type;`) re-parses cleanly.
///
/// ## 3. Preconditions
/// - `raw_type: &str` is a source-language type expression.
///
/// ## 4. Postconditions
/// - Returns a non-empty valid SysML v2 identifier string (`"String"`, `"Integer"`, `"Boolean"`, `"Real"`,
///   or a valid user/framework type identifier).
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) where N is `raw_type.len()`.
/// - Space Complexity: O(N) for the normalized identifier string.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Falls back to `"String"` when `raw_type` is empty or purely punctuation (`()`).
/// - State: Pure function.
/// - Action: Non-panicking prefix/suffix stripping and character filtering.
/// - Resource: Single `String` allocation.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-NORMALIZE-TYPE-SYSML]
pub fn normalize_type_for_sysml(raw_type: &str) -> String {
    let mut s = raw_type.trim();
    while let Some(stripped) = s.strip_suffix('?') {
        s = stripped.trim_end();
    }
    while let Some(stripped) = s.strip_prefix('&') {
        let mut r = stripped.trim_start();
        if r.starts_with('\'') {
            if let Some((_, after_space)) = r.split_once(' ') {
                r = after_space.trim_start();
            }
        }
        if let Some(after_mut) = r.strip_prefix("mut ") {
            r = after_mut.trim_start();
        }
        s = r;
    }
    if let Some(inner_slice) = s.strip_prefix('[').and_then(|r| r.strip_suffix(']')) {
        s = inner_slice
            .split_once(';')
            .map(|(elem, _)| elem.trim())
            .unwrap_or(inner_slice.trim());
    }

    if let Some((outer_raw, rest)) = s.split_once('<') {
        if let Some(inner_raw) = rest.strip_suffix('>') {
            let outer_last = outer_raw
                .trim()
                .rsplit_once("::")
                .map(|(_, last)| last.trim())
                .unwrap_or(outer_raw.trim());
            if is_unwrappable_generic_container(outer_last) {
                if let Some(first_arg) = split_top_level_comma(inner_raw)
                    .into_iter()
                    .find(|arg| !arg.trim().starts_with('\''))
                {
                    let trimmed_arg = first_arg.trim();
                    if !trimmed_arg.is_empty() && trimmed_arg != "()" && trimmed_arg != "void" {
                        return normalize_type_for_sysml(trimmed_arg);
                    }
                    return "String".to_string();
                }
            }
        }
    }

    let base = s
        .split_once('<')
        .map(|(outer, _)| outer.trim())
        .unwrap_or(s.trim());
    let last_segment = base.rsplit_once("::").map(|(_, last)| last).unwrap_or(base);
    let ident = sanitize_sysml_identifier(last_segment, "String");
    match ident.as_str() {
        "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8" | "i16" | "i32" | "i64" | "i128"
        | "isize" | "int" => "Integer".to_string(),
        "f32" | "f64" | "double" | "num" => "Real".to_string(),
        "bool" => "Boolean".to_string(),
        "str" | "String" | "char" | "Path" | "PathBuf" => "String".to_string(),
        "Self" | "Result" | "Option" | "Vec" | "Box" | "Arc" | "Rc" | "Future" | "Stream"
        | "void" | "dynamic" | "Object" | "Never" => "String".to_string(),
        other if is_sysml_reserved_keyword(other) => "String".to_string(),
        other => other.to_string(),
    }
}

/// Returns `true` if `outer_ident` is a generic wrapper or collection container whose primary payload type should be unwrapped during SysML v2 normalization (`#518`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for `is_unwrappable_generic_container`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Matches Rust and Dart generic wrapper, smart-pointer, asynchronous, and collection containers (`Result`, `Option`, `Vec`, `List`, `Future`, `Stream`, `ValueChanged`, `Arc`, `Mutex`, etc.).
///
/// ## 3. Preconditions
/// - `outer_ident: &str` is the unqualified outer generic type identifier preceding `'<'`.
///
/// ## 4. Postconditions
/// - Returns `true` iff `normalize_type_for_sysml` must recursively unwrap the first non-lifetime type argument.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) static token match.
/// - Space Complexity: O(1) zero heap allocation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Accepts any UTF-8 slice.
/// - State: Pure stateless predicate.
/// - Action: Zero panics or unwraps.
/// - Resource: Zero dynamic memory.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-NORMALIZE-TYPE-SYSML]
fn is_unwrappable_generic_container(outer_ident: &str) -> bool {
    matches!(
        outer_ident,
        "Result"
            | "Option"
            | "Vec"
            | "List"
            | "Set"
            | "Iterable"
            | "Box"
            | "Arc"
            | "Rc"
            | "Future"
            | "Stream"
            | "ValueChanged"
            | "ValueSetter"
            | "ValueGetter"
            | "EventSink"
            | "Sender"
            | "Receiver"
            | "Cow"
            | "Mutex"
            | "RwLock"
            | "RefCell"
            | "Cell"
            | "Pin"
    )
}

/// Checks whether an identifier collides with a reserved SysML v2 / KerML grammar keyword.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A SysML v2 serialization safety (`is_sysml_reserved_keyword`).
/// Prevents source identifiers like `state`, `part`, `port`, `action`, `in`, `out`, or `from`
/// from causing syntax errors when emitted into canonical SysML v2 text.
///
/// ## 2. Invariants & Mathematical Properties
/// - Returns `true` iff `ident` (case-insensitively) matches a reserved keyword token in `SysmlParser`.
///
/// ## 3. Preconditions
/// - `ident: &str` is a candidate identifier token.
///
/// ## 4. Postconditions
/// - Returns `true` if `ident` must be disambiguated before SysML v2 serialization, or `false` otherwise.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(K) where K is `ident.len()`.
/// - Space Complexity: O(K) for lowercase comparison.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Accepts any UTF-8 `&str`.
/// - State: Pure predicate.
/// - Action: Exhaustive keyword match.
/// - Resource: Single bounded lowercase string.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-SYSML-RESERVED-KEYWORD]
fn is_sysml_reserved_keyword(ident: &str) -> bool {
    let lower = ident.to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "package"
            | "part"
            | "def"
            | "port"
            | "attribute"
            | "action"
            | "state"
            | "transition"
            | "requirement"
            | "constraint"
            | "assert"
            | "satisfy"
            | "verify"
            | "item"
            | "flow"
            | "connection"
            | "connect"
            | "to"
            | "from"
            | "in"
            | "out"
            | "inout"
            | "use"
            | "case"
            | "calc"
            | "return"
            | "hazard"
            | "risk"
            | "capability"
            | "operation"
            | "interaction"
            | "import"
            | "doc"
            | "entry"
            | "do"
            | "exit"
            | "subject"
            | "actor"
            | "objective"
            | "step"
            | "perform"
            | "assume"
            | "require"
            | "true"
            | "false"
            | "null"
    )
}

/// Sanitizes an arbitrary string into a valid SysML v2 ASCII identifier `[A-Za-z_][A-Za-z0-9_]*`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and FEAT-42 canonical SysML v2 identifier hygiene (`sanitize_sysml_identifier`).
/// Guarantees that package, part, attribute, port, and action names emitted by `source_ast`
/// never violate SysML v2 lexical grammar.
///
/// ## 2. Invariants & Mathematical Properties
/// - Replaces non-alphanumeric characters with `'_'`, collapses consecutive `'_'`, and appends
///   `"_id"` if the resulting token collides with a reserved SysML v2 keyword.
///
/// ## 3. Preconditions
/// - `raw: &str` is the candidate name; `fallback: &str` is a valid non-empty identifier fallback.
///
/// ## 4. Postconditions
/// - Returns a non-empty valid SysML v2 identifier string.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) where N is `raw.len()`.
/// - Space Complexity: O(N) for the sanitized `String`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Returns `fallback.to_string()` when `raw` contains no alphanumeric characters.
/// - State: Pure string transformation.
/// - Action: Prefix-guards leading digits with `'_'`.
/// - Resource: Single `String` allocation.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-SANITIZE-SYSML-IDENT]
pub fn sanitize_sysml_identifier(raw: &str, fallback: &str) -> String {
    let mut out = String::new();
    let mut prev_underscore = false;
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            prev_underscore = false;
        } else if !prev_underscore && !out.is_empty() {
            out.push('_');
            prev_underscore = true;
        }
    }
    let trimmed = out.trim_matches('_');
    if trimmed.is_empty() {
        return fallback.to_string();
    }
    let mut result = if trimmed
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        format!("_{}", trimmed)
    } else {
        trimmed.to_string()
    };
    if is_sysml_reserved_keyword(&result) {
        result.push_str("_id");
    }
    result
}

/// Splits a comma-separated parameter or argument list at top level (ignoring commas inside `<>`, `()`, `[]`, `{}`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and Check 34 zero-regex parameter parsing (`split_top_level_comma`).
/// Splits parameter lists accurately even when parameter types contain nested generics like `Result<usize, String>`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Splits at byte index `i` iff `s.as_bytes().get(i) == Some(&b',')` and nesting depth of
///   `<>`, `()`, `[]`, and `{}` is zero.
///
/// ## 3. Preconditions
/// - `input: &str` is the interior of a parameter or argument list.
///
/// ## 4. Postconditions
/// - Returns `Vec<&str>` of non-empty top-level segments using `.get(..)`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) single-pass scan over `input`.
/// - Space Complexity: O(K) where K is the number of top-level comma-separated segments.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Uses `.get(..)` exclusively with saturating depth counters.
/// - State: Pure slice splitter.
/// - Action: Never panics on unbalanced inner brackets.
/// - Resource: Small vector of borrowed `&str` slices.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-SPLIT-TOP-LEVEL-COMMA]
fn split_top_level_comma(input: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth: usize = 0;
    let mut start: usize = 0;
    for (idx, ch) in input.char_indices() {
        match ch {
            '<' | '(' | '[' | '{' => {
                depth = depth.saturating_add(1);
            }
            '>' | ')' | ']' | '}' => {
                depth = depth.saturating_sub(1);
            }
            ',' if depth == 0 => {
                if let Some(seg) = input.get(start..idx) {
                    let trimmed = seg.trim();
                    if !trimmed.is_empty() {
                        parts.push(trimmed);
                    }
                }
                start = idx.saturating_add(1);
            }
            _ => {}
        }
    }
    if let Some(tail) = input.get(start..) {
        let trimmed = tail.trim();
        if !trimmed.is_empty() {
            parts.push(trimmed);
        }
    }
    parts
}

/// Extracts the leading ASCII identifier `[A-Za-z_][A-Za-z0-9_]*` from `input`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and Check 34 zero-regex identifier extraction (`extract_leading_ast_ident`).
/// Provides deterministic identifier tokenization across both Rust and Dart AST scanners.
///
/// ## 2. Invariants & Mathematical Properties
/// - Consumes the maximal prefix of ASCII alphanumeric or `'_'` characters starting with an ASCII letter or `'_'`.
///
/// ## 3. Preconditions
/// - `input: &str` is positioned at a candidate identifier token.
///
/// ## 4. Postconditions
/// - Returns `Some(String)` if `input` starts with `[A-Za-z_]`, or `None` otherwise.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(K) where K is the identifier character length.
/// - Space Complexity: O(K) for the returned `String`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Uses `.chars()` iterator without byte-offset indexing.
/// - State: Pure function.
/// - Action: Terminates at first non-identifier char.
/// - Resource: Single short `String` allocation.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-EXTRACT-LEADING-AST-IDENT]
pub fn extract_leading_ast_ident(input: &str) -> Option<String> {
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

/// Extracts the callable name preceding `'('` in a Dart function or method declaration line.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and Check 36 Dart callable detection parity (`extract_dart_callable_ident`).
/// Matches `<ReturnType> <callableName>(...)` while rejecting variable initializers where `'='` precedes `'('`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Strips optional trailing generic parameters `<T>` before the opening `'('`.
/// - Requires both a non-empty return type prefix and a valid identifier immediately before `'('`.
///
/// ## 3. Preconditions
/// - `code: &str` is a trimmed, modifier-stripped Dart line at `brace_depth <= 1`.
///
/// ## 4. Postconditions
/// - Returns `Some(callable_name)` for Dart function/method declarations, or `None`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) where N is `code.len()`.
/// - Space Complexity: O(I) where I is the length of `callable_name`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Non-panicking `.get(..)` and reverse character iteration.
/// - State: Pure lexical helper.
/// - Action: Never panics on malformed input.
/// - Resource: Bounded by single line length.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-EXTRACT-DART-CALLABLE-IDENT]
pub fn extract_dart_callable_ident(code: &str) -> Option<String> {
    let (before_paren, _) = code.split_once('(')?;
    if before_paren.contains('=') || before_paren.contains(';') {
        return None;
    }
    let trimmed_before = before_paren.trim();
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
            trimmed_before
                .get(..idx)
                .unwrap_or(trimmed_before)
                .trim_end()
        } else {
            trimmed_before
        }
    } else {
        trimmed_before
    };

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

/// Determines the byte length of a single-character literal at the start of `slice`.
///
/// # Summary
/// Computes the exact byte length of a Rust or Dart single-character literal (`'a'`, `'\n'`, `'\x7F'`,
/// `'\u{1F600}'`, `'↑'`) starting at byte 0 of `slice`, or returns `None` for lifetimes (`'a`, `'static`)
/// and multi-character single-quoted strings.
///
/// # Preconditions
/// - `slice` is a valid UTF-8 string slice positioned at a candidate `'` token.
///
/// # Postconditions
/// - Returns `Some(byte_len)` with `byte_len >= 3` iff `slice` starts with a well-formed character literal; `None` otherwise.
///
/// # Invariants
/// - Never uses magic byte-offset limits; inspects the exact UTF-8 scalar or escape sequence followed by `'`.
/// - Uses non-panicking `.get(..)` and `.chars()` exclusively.
///
/// # Failure Modes
/// - Unclosed single quotes, empty `''`, or lifetime identifiers (`'lifetime_alpha`) return `None`.
///
/// # Side Effects
/// - Pure stateless lexical recognizer; zero I/O or mutation.
///
/// # Determinism
/// - Deterministic O(1) bounded inspection of the leading character literal token.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A character literal parsing standards.
/// Correctly computes byte lengths for ASCII, escape-sequence, and multi-byte UTF-8 character literals.
///
/// ## 2. Invariants & Mathematical Properties
/// - Returns `Some(byte_length)` if `slice` starts with a well-formed character literal, or `None`.
/// - Supports multi-byte unicode code points up to 4 UTF-8 bytes and `\u{...}` hex escapes.
///
/// ## 3. Preconditions
/// - `slice` is a valid UTF-8 string slice.
///
/// ## 4. Postconditions
/// - Returns `Some(len)` where `len >= 3` and matches the full character literal token.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) bounded inspection of leading bytes.
/// - Space Complexity: O(1) zero allocation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Non-panicking string slicing using `.get(..)`.
/// - State: Stateless.
/// - Action: Rejects unclosed or empty single-quoted tokens safely.
/// - Resource: Zero heap allocations.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SOURCE-AST-SINGLE-CHAR-LITERAL]
pub fn single_char_literal_byte_len(slice: &str) -> Option<usize> {
    if !slice.starts_with('\'') {
        return None;
    }
    let rest = slice.get(1..)?;
    if rest.starts_with('\\') {
        let after_backslash = rest.get(1..)?;
        let mut chars = after_backslash.chars();
        let first = chars.next()?;
        if first == 'x' {
            if after_backslash.get(3..4) == Some("'") {
                return Some(6);
            }
        } else if first == 'u' {
            if let Some(close_brace) = after_backslash.find('}') {
                if after_backslash.get(close_brace.saturating_add(1)..close_brace.saturating_add(2))
                    == Some("'")
                {
                    return Some(close_brace.saturating_add(4));
                }
            }
        } else if after_backslash.get(first.len_utf8()..first.len_utf8().saturating_add(1))
            == Some("'")
        {
            return Some(first.len_utf8().saturating_add(3));
        }
        None
    } else {
        let mut chars = rest.chars();
        let ch = chars.next()?;
        if ch != '\''
            && ch != '\\'
            && rest.get(ch.len_utf8()..ch.len_utf8().saturating_add(1)) == Some("'")
        {
            Some(ch.len_utf8().saturating_add(2))
        } else {
            None
        }
    }
}

/// Multi-line lexical scanner state for Rust source files.
///
/// # Summary
/// Tracks whether the line-oriented Rust scanner is in normal code, a nested `/* ... */` block comment,
/// a standard `"..."` string literal, or a `r#"..."#` raw string literal with `hashes` `#` delimiters.
///
/// # Preconditions
/// - Initialized to [`RustStringScannerState::Normal`] at line 1 of a Rust source file.
///
/// # Postconditions
/// - Updated in place by [`scan_rust_line_preserving_lex_state`] across every source line.
///
/// # Invariants
/// - `BlockComment(depth)` always has `depth >= 1` while active and transitions to `Normal` when `depth == 0`.
/// - `RawString(hashes)` closes only when `"` is immediately followed by `hashes` consecutive `#` bytes.
///
/// # Failure Modes
/// - Unclosed literals or comments remain in their active non-`Normal` state through EOF.
///
/// # Side Effects
/// - Pure value enum with `Copy` semantics; zero heap allocations.
///
/// # Determinism
/// - Deterministic state transitions driven strictly by UTF-8 byte sequences.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and Check 34/Check 36 unified Rust lexical state machine rules (`RustStringScannerState`).
/// Shared across `deap-core::source_ast` and `verify-baseline` to guarantee identical multi-line comment,
/// string literal, raw string, character literal, and lifetime tokenization.
///
/// ## 2. Invariants & Mathematical Properties
/// - Four mutually exclusive states: `Normal`, `BlockComment(usize)`, `StringLiteral`, and `RawString(usize)`.
///
/// ## 3. Preconditions
/// - Passed by mutable reference into [`scan_rust_line_preserving_lex_state`].
///
/// ## 4. Postconditions
/// - Reflects the exact lexical mode at the end of each scanned line.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) state transition and comparison.
/// - Space Complexity: O(1) stack footprint.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Total state representation with zero panicking transitions.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SOURCE-AST-RUST-SCANNER-STATE]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RustStringScannerState {
    /// Outside any comment or string literal.
    Normal,
    /// Inside a nested `/* ... */` block comment of the given nesting depth (`>= 1`).
    BlockComment(usize),
    /// Inside a standard double-quoted `"..."` string literal.
    StringLiteral,
    /// Inside a raw string literal `r#"..."#` with the specified number of `#` delimiters.
    RawString(usize),
}

/// Scans a single Rust source line, blanking out comment and string/char literal interiors while updating `state`.
///
/// # Summary
/// Transforms `line` into a sanitized code string in which `//` comments and `/* ... */` block comments are
/// stripped and the interior bytes of `"..."`, `r#"..."#`, and `'c'` character literals are replaced with spaces,
/// while preserving Rust lifetimes (`'a`, `'static`, `'lifetime_alpha`) and code tokens that follow a mid-line
/// `*/` or closing string quote.
///
/// # Preconditions
/// - `line` is a single UTF-8 line slice (without trailing newline).
/// - `state` holds the [`RustStringScannerState`] carried over from the preceding line.
///
/// # Postconditions
/// - Returns a `String` containing only top-level code tokens and placeholder quotes/spaces for literals.
/// - Updates `*state` to the lexical state at the end of `line`.
///
/// # Invariants
/// - Character literals (`'{'`, `'/'`, `'\''`, `'\u{1F600}'`) are recognized via [`single_char_literal_byte_len`]
///   and blanked out with spaces so they never corrupt brace, parenthesis, or comment scanning.
/// - Lifetimes (`'a`, `'static`, `'lifetime_alpha`) advance by 1 byte (`'`) without entering a string state.
///
/// # Failure Modes
/// - Malformed or unclosed literals on `line` transition `*state` without panicking or out-of-bounds slicing.
///
/// # Side Effects
/// - Mutates `*state` in place; allocates and returns one sanitized `String`.
///
/// # Determinism
/// - Deterministic O(N) single-pass scanner over `line`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and Check 34/Check 36 Rust lexical hygiene (`scan_rust_line_preserving_lex_state`).
/// Prevents `//`, `/*`, `{`, `}`, `(`, `)`, or `fn ` inside strings, raw strings, comments, or character
/// literals from corrupting AST extraction or baseline compliance checks.
///
/// ## 2. Invariants & Mathematical Properties
/// - Handles nested `/* ... /* ... */ ... */` block comments via `BlockComment(depth)`.
/// - Handles arbitrary-hash raw strings `r"..."`, `r#"..."#`, `r##"..."##` via `RawString(hashes)`.
/// - Distinguishes character literals from lifetimes of any identifier length via [`single_char_literal_byte_len`].
///
/// ## 3. Preconditions
/// - `line: &str` is a valid UTF-8 Rust source line; `state: &mut RustStringScannerState` is the active state.
///
/// ## 4. Postconditions
/// - Returns the sanitized code string for `line` and updates `*state`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) where N is `line.len()`.
/// - Space Complexity: O(N) for the returned sanitized `String`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Non-panicking `.get(..)` and saturating index arithmetic throughout.
/// - State: Mutates only `*state`.
/// - Action: Preserves code tokens appearing after a mid-line `*/` block comment closure.
/// - Resource: Single `String` allocation with capacity `line.len()`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SOURCE-AST-SCAN-RUST-LINE-LEX-STATE]
pub fn scan_rust_line_preserving_lex_state(
    line: &str,
    state: &mut RustStringScannerState,
) -> String {
    let bytes = line.as_bytes();
    let mut out = String::with_capacity(line.len());
    let mut i: usize = 0;

    while i < bytes.len() {
        match *state {
            RustStringScannerState::BlockComment(mut depth) => {
                if bytes.get(i) == Some(&b'/') && bytes.get(i.saturating_add(1)) == Some(&b'*') {
                    depth = depth.saturating_add(1);
                    *state = RustStringScannerState::BlockComment(depth);
                    i = i.saturating_add(2);
                } else if bytes.get(i) == Some(&b'*')
                    && bytes.get(i.saturating_add(1)) == Some(&b'/')
                {
                    depth = depth.saturating_sub(1);
                    *state = if depth == 0 {
                        RustStringScannerState::Normal
                    } else {
                        RustStringScannerState::BlockComment(depth)
                    };
                    out.push(' ');
                    i = i.saturating_add(2);
                } else {
                    i = i.saturating_add(1);
                }
            }
            RustStringScannerState::StringLiteral => {
                if bytes.get(i) == Some(&b'\\') {
                    out.push(' ');
                    if i.saturating_add(1) < bytes.len() {
                        out.push(' ');
                        i = i.saturating_add(2);
                    } else {
                        i = i.saturating_add(1);
                    }
                } else if bytes.get(i) == Some(&b'"') {
                    *state = RustStringScannerState::Normal;
                    out.push('"');
                    i = i.saturating_add(1);
                } else {
                    out.push(' ');
                    i = i.saturating_add(1);
                }
            }
            RustStringScannerState::RawString(hashes) => {
                if bytes.get(i) == Some(&b'"') {
                    let mut matched: usize = 0;
                    while matched < hashes
                        && bytes.get(i.saturating_add(1).saturating_add(matched)) == Some(&b'#')
                    {
                        matched = matched.saturating_add(1);
                    }
                    if matched == hashes {
                        *state = RustStringScannerState::Normal;
                        out.push('"');
                        i = i.saturating_add(1).saturating_add(hashes);
                        continue;
                    }
                }
                out.push(' ');
                i = i.saturating_add(1);
            }
            RustStringScannerState::Normal => {
                if bytes.get(i) == Some(&b'/') && bytes.get(i.saturating_add(1)) == Some(&b'/') {
                    break;
                }
                if bytes.get(i) == Some(&b'/') && bytes.get(i.saturating_add(1)) == Some(&b'*') {
                    *state = RustStringScannerState::BlockComment(1);
                    out.push(' ');
                    i = i.saturating_add(2);
                    continue;
                }
                if is_valid_raw_string_prefix(bytes, i, true) {
                    let mut hashes: usize = 0;
                    while bytes.get(i.saturating_add(1).saturating_add(hashes)) == Some(&b'#') {
                        hashes = hashes.saturating_add(1);
                    }
                    if bytes.get(i.saturating_add(1).saturating_add(hashes)) == Some(&b'"') {
                        *state = RustStringScannerState::RawString(hashes);
                        out.push('"');
                        i = i.saturating_add(2).saturating_add(hashes);
                        continue;
                    }
                }
                if bytes.get(i) == Some(&b'"') {
                    *state = RustStringScannerState::StringLiteral;
                    out.push('"');
                    i = i.saturating_add(1);
                    continue;
                }
                if bytes.get(i) == Some(&b'\'') {
                    if let Some(char_len) = line.get(i..).and_then(single_char_literal_byte_len) {
                        for _ in 0..char_len {
                            out.push(' ');
                        }
                        i = i.saturating_add(char_len);
                        continue;
                    }
                    out.push('\'');
                    i = i.saturating_add(1);
                    continue;
                }
                if let Some(ch) = line.get(i..).and_then(|s| s.chars().next()) {
                    out.push(ch);
                    i = i.saturating_add(ch.len_utf8());
                } else {
                    i = i.saturating_add(1);
                }
            }
        }
    }

    out
}

/// Scans a single Rust or Dart source line with unified [`RustDartLexState`] tracking (`REQ-0605` `AC-01..AC-02`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for `scan_line_with_unified_lex_state` (`REQ-0605` `AC-01..AC-02`).
///
/// ## 2. Invariants & Mathematical Properties
/// - Maintains `block_comment_depth`, `raw_hash_count`, `active_string`, and nested `interp_stack` (`Vec<InterpolationFrame>`) across lines.
/// - Strips `//` line comments and `/* ... */` block comments while blanking string and character literal interiors with spaces.
/// - Preserves Rust lifetimes (`'a`, `'static`) in Rust mode while handling single-quoted, double-quoted, triple-quoted, raw, and `${...}` interpolated strings in Dart mode.
///
/// ## 3. Preconditions
/// - `line: &str` is a single UTF-8 source line slice; `state: &mut RustDartLexState` carries the active file's lexical state.
///
/// ## 4. Postconditions
/// - Returns the sanitized code string for `line` and updates `*state` in place.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) where N is `line.len()`.
/// - Space Complexity: O(N) for the returned sanitized `String`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Non-panicking `.get(..)` byte access and saturating arithmetic throughout.
/// - State: Mutates only `*state`.
/// - Action: Zero panics, unwraps, expectations, or raw slice indexing.
/// - Resource: Bounded single `String` allocation of capacity `line.len()`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0605], [REQ-SOURCE-AST-SCAN-RUST-LINE-LEX-STATE]
pub fn scan_line_with_unified_lex_state(line: &str, state: &mut RustDartLexState) -> String {
    let bytes = line.as_bytes();
    let mut out = String::with_capacity(line.len());
    let mut i: usize = 0;

    while i < bytes.len() {
        if state.block_comment_depth > 0 {
            if bytes.get(i) == Some(&b'/') && bytes.get(i.saturating_add(1)) == Some(&b'*') {
                state.block_comment_depth = state.block_comment_depth.saturating_add(1);
                i = i.saturating_add(2);
            } else if bytes.get(i) == Some(&b'*') && bytes.get(i.saturating_add(1)) == Some(&b'/') {
                state.block_comment_depth = state.block_comment_depth.saturating_sub(1);
                out.push(' ');
                i = i.saturating_add(2);
            } else {
                i = i.saturating_add(1);
            }
            continue;
        }

        if let Some(hashes) = state.raw_hash_count {
            if bytes.get(i) == Some(&b'"') {
                let mut matched: usize = 0;
                while matched < hashes
                    && bytes.get(i.saturating_add(1).saturating_add(matched)) == Some(&b'#')
                {
                    matched = matched.saturating_add(1);
                }
                if matched == hashes {
                    state.raw_hash_count = None;
                    out.push('"');
                    i = i.saturating_add(1).saturating_add(hashes);
                    continue;
                }
            }
            out.push(' ');
            i = i.saturating_add(1);
            continue;
        }

        if let Some((quote, is_triple, is_raw)) = state.active_string {
            if !is_raw && bytes.get(i) == Some(&b'\\') {
                out.push(' ');
                if i.saturating_add(1) < bytes.len() {
                    out.push(' ');
                    i = i.saturating_add(2);
                } else {
                    i = i.saturating_add(1);
                }
                continue;
            }
            if state.language != SourceLanguage::Rust
                && !is_raw
                && bytes.get(i) == Some(&b'$')
                && bytes.get(i.saturating_add(1)) == Some(&b'{')
                && (state.language == SourceLanguage::Dart
                    || is_triple
                    || has_matching_interpolation_brace_before_quote(
                        bytes,
                        i.saturating_add(2),
                        quote,
                    ))
            {
                state.interp_stack.push(InterpolationFrame {
                    quote_byte: quote,
                    is_triple,
                    is_raw,
                    brace_depth: 1,
                });
                state.active_string = None;
                out.push(' ');
                out.push(' ');
                i = i.saturating_add(2);
                continue;
            }
            if is_triple {
                if bytes.get(i) == Some(&quote)
                    && bytes.get(i.saturating_add(1)) == Some(&quote)
                    && bytes.get(i.saturating_add(2)) == Some(&quote)
                {
                    state.active_string = None;
                    if state.interp_stack.is_empty() {
                        out.push(char::from(quote));
                    } else {
                        out.push(' ');
                    }
                    i = i.saturating_add(3);
                    continue;
                }
            } else if bytes.get(i) == Some(&quote) {
                state.active_string = None;
                if state.interp_stack.is_empty() {
                    out.push(char::from(quote));
                } else {
                    out.push(' ');
                }
                i = i.saturating_add(1);
                continue;
            }
            out.push(' ');
            i = i.saturating_add(1);
            continue;
        }

        if bytes.get(i) == Some(&b'/') && bytes.get(i.saturating_add(1)) == Some(&b'/') {
            break;
        }
        if bytes.get(i) == Some(&b'/') && bytes.get(i.saturating_add(1)) == Some(&b'*') {
            state.block_comment_depth = 1;
            out.push(' ');
            i = i.saturating_add(2);
            continue;
        }

        if bytes.get(i) == Some(&b'{') && !state.interp_stack.is_empty() {
            if let Some(frame) = state.interp_stack.last_mut() {
                frame.brace_depth = frame.brace_depth.saturating_add(1);
            }
            out.push(' ');
            i = i.saturating_add(1);
            continue;
        }
        if bytes.get(i) == Some(&b'}') && !state.interp_stack.is_empty() {
            let mut closed_interp = false;
            if let Some(frame) = state.interp_stack.last_mut() {
                frame.brace_depth = frame.brace_depth.saturating_sub(1);
                if frame.brace_depth == 0 {
                    closed_interp = true;
                }
            }
            if closed_interp {
                if let Some(popped) = state.interp_stack.pop() {
                    state.active_string =
                        Some((popped.quote_byte, popped.is_triple, popped.is_raw));
                }
            }
            out.push(' ');
            i = i.saturating_add(1);
            continue;
        }

        if state.language != SourceLanguage::Dart && is_valid_raw_string_prefix(bytes, i, true) {
            let mut hashes: usize = 0;
            while bytes.get(i.saturating_add(1).saturating_add(hashes)) == Some(&b'#') {
                hashes = hashes.saturating_add(1);
            }
            if bytes.get(i.saturating_add(1).saturating_add(hashes)) == Some(&b'"') {
                state.raw_hash_count = Some(hashes);
                out.push('"');
                i = i.saturating_add(2).saturating_add(hashes);
                continue;
            }
        }

        if state.language != SourceLanguage::Rust && is_valid_raw_string_prefix(bytes, i, false) {
            let next_idx = i.saturating_add(1);
            if let Some(&q) = bytes.get(next_idx) {
                if q == b'"' || q == b'\'' {
                    let is_triple = bytes.get(next_idx.saturating_add(1)) == Some(&q)
                        && bytes.get(next_idx.saturating_add(2)) == Some(&q);
                    state.active_string = Some((q, is_triple, true));
                    if state.interp_stack.is_empty() {
                        out.push(char::from(q));
                    } else {
                        out.push(' ');
                    }
                    i = next_idx.saturating_add(if is_triple { 3 } else { 1 });
                    continue;
                }
            }
        }

        if bytes.get(i) == Some(&b'"') {
            let is_triple = state.language != SourceLanguage::Rust
                && bytes.get(i.saturating_add(1)) == Some(&b'"')
                && bytes.get(i.saturating_add(2)) == Some(&b'"');
            state.active_string = Some((b'"', is_triple, false));
            if state.interp_stack.is_empty() {
                out.push('"');
            } else {
                out.push(' ');
            }
            i = i.saturating_add(if is_triple { 3 } else { 1 });
            continue;
        }

        if bytes.get(i) == Some(&b'\'') {
            if state.language != SourceLanguage::Dart {
                if let Some(char_len) = line.get(i..).and_then(single_char_literal_byte_len) {
                    for _ in 0..char_len {
                        out.push(' ');
                    }
                    i = i.saturating_add(char_len);
                    continue;
                }
                if state.language == SourceLanguage::Rust || is_rust_lifetime_at(bytes, i) {
                    out.push('\'');
                    i = i.saturating_add(1);
                    continue;
                }
            }
            let is_triple = bytes.get(i.saturating_add(1)) == Some(&b'\'')
                && bytes.get(i.saturating_add(2)) == Some(&b'\'');
            state.active_string = Some((b'\'', is_triple, false));
            if state.interp_stack.is_empty() {
                out.push('\'');
            } else {
                out.push(' ');
            }
            i = i.saturating_add(if is_triple { 3 } else { 1 });
            continue;
        }

        if !state.interp_stack.is_empty() {
            out.push(' ');
            i = i.saturating_add(1);
            continue;
        }

        if let Some(ch) = line.get(i..).and_then(|s| s.chars().next()) {
            out.push(ch);
            i = i.saturating_add(ch.len_utf8());
        } else {
            i = i.saturating_add(1);
        }
    }

    out
}

/// Validates whether byte index `r_idx` in `bytes` is a standalone raw string prefix (`r`/`R` or `br`/`BR`).
///
/// # Summary
/// Checks that `bytes.get(r_idx)` is `b'r'` or `b'R'` and is either at the start of `bytes`, preceded by a
/// non-identifier byte (`![A-Za-z0-9_]`), or (when `allow_byte_prefix` is `true`) preceded by `b'b'` / `b'B'`
/// which is itself at index 0 or preceded by a non-identifier byte.
///
/// # Preconditions
/// - `bytes` is a UTF-8 source or line byte slice; `r_idx` is the candidate index of `'r'` or `'R'`.
///
/// # Postconditions
/// - Returns `true` iff `r_idx` is a valid raw string (or raw byte string) prefix token boundary.
///
/// # Invariants
/// - Rejects identifier suffixes ending in `r` or `br` (such as `bar"..."` or `subr"..."`).
///
/// # Failure Modes
/// - Returns `false` when `r_idx` is out of bounds or preceded by an identifier character.
///
/// # Side Effects
/// - Pure stateless predicate; zero I/O or heap allocation.
///
/// # Determinism
/// - Deterministic O(1) bounded byte lookbehind.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for `is_valid_raw_string_prefix`.
///
/// ## 2. Invariants & Mathematical Properties
/// - `r_idx == 0` or predecessor is non-identifier, or (when `allow_byte_prefix`) predecessor is `b`/`B` with non-identifier before it.
///
/// ## 3. Preconditions
/// - `bytes: &[u8]` is a valid byte slice; `r_idx: usize` is a byte offset; `allow_byte_prefix: bool` controls Rust `br` acceptance.
///
/// ## 4. Postconditions
/// - Returns `true` on a valid raw string prefix boundary and `false` otherwise.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) constant-time 2-byte lookbehind.
/// - Space Complexity: O(1) zero allocation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Non-panicking `.get(..)` byte inspection with saturating subtraction.
/// - State: Stateless.
/// - Action: Zero panics, unwraps, or slice indexing.
/// - Resource: Zero heap allocations.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SOURCE-AST-RAW-STRING-HASH-COUNT]
fn is_valid_raw_string_prefix(bytes: &[u8], r_idx: usize, allow_byte_prefix: bool) -> bool {
    if !matches!(bytes.get(r_idx), Some(b'r' | b'R')) {
        return false;
    }
    if r_idx == 0 {
        return true;
    }
    let prev = match bytes.get(r_idx.saturating_sub(1)) {
        Some(&p) => p,
        None => return true,
    };
    if !prev.is_ascii_alphanumeric() && prev != b'_' {
        return true;
    }
    if allow_byte_prefix && (prev == b'b' || prev == b'B') {
        if r_idx < 2 {
            return true;
        }
        return bytes
            .get(r_idx.saturating_sub(2))
            .map(|&p2| !p2.is_ascii_alphanumeric() && p2 != b'_')
            .unwrap_or(true);
    }
    false
}

/// Detects a Rust hashed raw string prefix `r#"..."#` (or `br#"..."#`) at byte offset `idx`.
///
/// # Summary
/// Inspects `bytes` at `idx` for an `'r'` or `'R'` token (validated via [`is_valid_raw_string_prefix`])
/// followed by `hashes >= 1` `'#'` bytes and an opening `'"'`, returning `Some((hashes, quote_idx))`.
///
/// # Preconditions
/// - `bytes` is a source line byte slice; `idx` is the candidate byte index of `'r'` or `'R'`.
///
/// # Postconditions
/// - Returns `Some((hashes, quote_idx))` where `hashes >= 1` and `bytes.get(quote_idx) == Some(&b'"')`, or `None`.
///
/// # Invariants
/// - Rejects identifier suffixes ending in `r` or `br` (such as `bar#"..."` or `subr#"..."`).
///
/// # Failure Modes
/// - Returns `None` when `idx` is out of bounds, lacks `#`, or is not followed by `'"'`.
///
/// # Side Effects
/// - Pure stateless helper; zero I/O or heap allocation.
///
/// # Determinism
/// - Deterministic O(H) scan where H is the number of consecutive `#` characters.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and Check 34 raw string lexical recognition (`rust_raw_string_hash_count`).
/// Enables `strip_line_comment_preserving_strings` and `update_ast_delimiter_depths` to preserve `//`
/// and unbalanced delimiters inside `r#"..."#` and `br#"..."#` literals.
///
/// ## 2. Invariants & Mathematical Properties
/// - `quote_idx == idx + 1 + hashes` with `hashes >= 1`.
///
/// ## 3. Preconditions
/// - `bytes: &[u8]` is a UTF-8 line byte slice; `idx: usize` is a byte offset.
///
/// ## 4. Postconditions
/// - Returns `Some((hashes, quote_idx))` on valid hashed raw-string opening, or `None`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(H) where H is the hash delimiter count.
/// - Space Complexity: O(1) zero allocation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-panicking `.get(..)` byte inspection with saturating index addition.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SOURCE-AST-RAW-STRING-HASH-COUNT]
fn rust_raw_string_hash_count(bytes: &[u8], idx: usize) -> Option<(usize, usize)> {
    let b = *bytes.get(idx)?;
    if (b != b'r' && b != b'R') || !is_valid_raw_string_prefix(bytes, idx, true) {
        return None;
    }
    let mut scan = idx.saturating_add(1);
    let mut hashes: usize = 0;
    while bytes.get(scan) == Some(&b'#') {
        hashes = hashes.saturating_add(1);
        scan = scan.saturating_add(1);
    }
    if hashes > 0 && bytes.get(scan) == Some(&b'"') {
        Some((hashes, scan))
    } else {
        None
    }
}

/// Verifies whether a `${` sequence inside a quoted string literal has a matching `}` before the closing `quote`.
///
/// # Summary
/// Scans forward from `start_idx` (immediately after `${`) to confirm that a matching closing `}`
/// (accounting for nested braces and nested string literals inside `${...}`) appears before the outer
/// string's closing `quote` on the same line.
///
/// # Preconditions
/// - `bytes` is the byte slice of a source line.
/// - `start_idx` is the byte index immediately following `${` (`idx.saturating_add(2)`).
/// - `quote` is the active string delimiter (`b'"'` or `b'\''`).
///
/// # Postconditions
/// - Returns `true` iff `${` is closed by a matching `}` followed by an unescaped closing `quote` on the line.
/// - Returns `false` for unclosed `"${"` literals (such as Rust string literals containing literal `"${"`).
///
/// # Invariants
/// - Preserves Dart interpolated strings with nested quotes (`"hello ${map["world"]}!"`) while rejecting
///   Rust string literals containing literal `"${"` without a closing `}` before `"`.
///
/// # Failure Modes
/// - Unclosed braces or quotes return `false` at `bytes.len()` without panicking.
///
/// # Side Effects
/// - Pure stateless lookahead helper; zero I/O or heap allocation.
///
/// # Determinism
/// - Deterministic O(N) byte lookahead over the remainder of the line.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A delimiter depth soundness (`has_matching_interpolation_brace_before_quote`).
/// Prevents literal `"${"` substrings inside Rust string literals from exiting quote state in
/// `update_ast_delimiter_depths` and corrupting `brace_depth` or `paren_depth`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Tracks brace nesting `depth` starting at `1` and optional `inner_quote` inside `${...}`.
/// - Requires `depth` to reach `0` outside `inner_quote` with a valid closing `quote` after the `}`.
///
/// ## 3. Preconditions
/// - `bytes: &[u8]`, `start_idx: usize`, `quote: u8`.
///
/// ## 4. Postconditions
/// - Returns `true` only when a complete `${...}` interpolation block exists inside the quoted literal.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) where N is `bytes.len() - start_idx`.
/// - Space Complexity: O(1) zero heap allocation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-panicking `.get(..)` indexing and saturating arithmetic.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SOURCE-AST-INTERP-BRACE-LOOKAHEAD]
fn has_matching_interpolation_brace_before_quote(
    bytes: &[u8],
    start_idx: usize,
    quote: u8,
) -> bool {
    let mut i = start_idx;
    let mut depth: usize = 1;
    let mut inner_quote: Option<u8> = None;
    let mut last_non_ws: Option<u8> = None;

    while let Some(&b) = bytes.get(i) {
        if let Some(iq) = inner_quote {
            if b == b'\\' {
                i = i.saturating_add(2);
                continue;
            }
            if b == iq {
                inner_quote = None;
                last_non_ws = Some(b);
            }
            i = i.saturating_add(1);
            continue;
        }

        if b == b'\\' {
            i = i.saturating_add(2);
            continue;
        }
        if b == b'{' {
            depth = depth.saturating_add(1);
            last_non_ws = Some(b);
        } else if b == b'}' {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return has_unescaped_closing_quote(bytes, i.saturating_add(1), quote);
            }
            last_non_ws = Some(b);
        } else if b == quote {
            let can_open_inner_quote = matches!(
                last_non_ws,
                Some(
                    b'(' | b'['
                        | b'{'
                        | b','
                        | b':'
                        | b'?'
                        | b'='
                        | b'+'
                        | b'-'
                        | b'*'
                        | b'/'
                        | b'%'
                        | b'&'
                        | b'|'
                        | b'^'
                        | b'!'
                        | b'<'
                        | b'>'
                        | b'~'
                )
            );
            if can_open_inner_quote {
                inner_quote = Some(b);
                last_non_ws = Some(b);
            } else {
                return false;
            }
        } else if b == b'"' || b == b'\'' {
            inner_quote = Some(b);
            last_non_ws = Some(b);
        } else if !b.is_ascii_whitespace() {
            last_non_ws = Some(b);
        }

        i = i.saturating_add(1);
    }

    false
}

/// Checks whether an unescaped closing quote character exists on the remainder of the line.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A string literal boundary verification.
/// Ensures single-line string tokenization only commences when a terminating matching quote is present.
///
/// ## 2. Invariants & Mathematical Properties
/// - Scans byte-by-byte skipping backslash-escaped characters.
/// - Returns `true` if `quote` is encountered without an unescaped preceding backslash.
///
/// ## 3. Preconditions
/// - `bytes` is a byte slice; `start_idx` is a valid search start offset.
///
/// ## 4. Postconditions
/// - Returns a deterministic boolean indicating unescaped quote presence.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) where N is remaining slice length.
/// - Space Complexity: O(1) zero allocation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Non-panicking index loop bounded by `bytes.len()`.
/// - State: Stateless.
/// - Action: Never panics on boundary conditions.
/// - Resource: Zero heap allocations.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SOURCE-AST-CLOSING-QUOTE-CHECK]
fn has_unescaped_closing_quote(bytes: &[u8], start_idx: usize, quote: u8) -> bool {
    let mut i = start_idx;
    while i < bytes.len() {
        if bytes.get(i) == Some(&b'\\') {
            i = i.saturating_add(2);
            continue;
        }
        if bytes.get(i) == Some(&quote) {
            return true;
        }
        i = i.saturating_add(1);
    }
    false
}

/// Disambiguates Rust lifetime annotations (`&'a`, `<'a>`, `'a: 'b`) from character or string literals.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A Rust grammar tokenization rules.
/// Prevents lifetime identifiers from being erroneously tokenized as unclosed character literals.
///
/// ## 2. Invariants & Mathematical Properties
/// - Returns `true` if the single quote at `idx` is immediately followed by a Rust lifetime identifier
///   and is preceded by lifetime-introducing punctuation (`&`, `<`, `,`, `:`, `(`).
/// - Returns `false` if followed immediately by a closing single quote (character literal).
///
/// ## 3. Preconditions
/// - `bytes` is a byte slice of a source line; `idx` is the offset of a single quote `b'\''`.
///
/// ## 4. Postconditions
/// - Returns `true` if `idx` represents a Rust lifetime, `false` otherwise.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(K) where K is the length of the lifetime identifier.
/// - Space Complexity: O(1) zero allocation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Safe indexing with `.get(..)` and saturating arithmetic.
/// - State: Stateless predicate.
/// - Action: Never panics.
/// - Resource: Zero allocations.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SOURCE-AST-RUST-LIFETIME-CHECK]
fn is_rust_lifetime_at(bytes: &[u8], idx: usize) -> bool {
    if bytes.get(idx) != Some(&b'\'') {
        return false;
    }
    match bytes.get(idx.saturating_add(1)) {
        Some(&b) if b.is_ascii_alphabetic() || b == b'_' => {}
        _ => return false,
    }
    let mut end = idx.saturating_add(2);
    while let Some(&b) = bytes.get(end) {
        if b.is_ascii_alphanumeric() || b == b'_' {
            end = end.saturating_add(1);
        } else {
            break;
        }
    }
    if bytes.get(end) == Some(&b'\'') {
        return false;
    }

    let mut prev = idx;
    while prev > 0
        && bytes
            .get(prev.saturating_sub(1))
            .map(|b| b.is_ascii_whitespace())
            .unwrap_or(false)
    {
        prev = prev.saturating_sub(1);
    }
    let prev_byte = if prev > 0 {
        bytes.get(prev.saturating_sub(1)).copied()
    } else {
        None
    };

    matches!(prev_byte, Some(b'&' | b'<' | b',' | b'(' | b':')) || bytes.get(end) == Some(&b':')
}

/// Strips a line comment (`// ...`) from a source line while preserving `//` inside string and char literals.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and Check 36 comment-stripping rules.
/// Prevents URL protocols (`http://`, `package://`) and comments inside standard, interpolated (`${...}`),
/// or hashed raw string literals (`r#"..."#`) from being prematurely truncated as line comments in Rust and Dart source files.
///
/// ## 2. Invariants & Mathematical Properties
/// - Only truncates at `//` when neither `active_quote` nor `raw_hashes` is active.
/// - Accurately skips escaped quotes, hashed raw strings `r#"..."#`, nested `${...}` interpolations, and character literals.
///
/// ## 3. Preconditions
/// - `line` is a valid UTF-8 string slice.
///
/// ## 4. Postconditions
/// - Returns a prefix slice of `line` up to the first unquoted `//`, or the entire `line` if none exists.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(L) where L is byte length of `line`.
/// - Space Complexity: O(D) where D is maximum string interpolation nesting depth.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-panicking byte scanner using `.get(..)`.
/// - Stateless pure function.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0605], [REQ-SOURCE-AST-STRIP-LINE-COMMENT]
pub fn strip_line_comment_preserving_strings(line: &str) -> &str {
    let bytes = line.as_bytes();
    let mut idx: usize = 0;
    let mut active_quote: Option<u8> = None;
    let mut is_raw = false;
    let mut raw_hashes: Option<usize> = None;
    let mut brace_depth: usize = 0;
    let mut interp_stack: Vec<(u8, bool, usize)> = Vec::new();

    while let Some(&b) = bytes.get(idx) {
        if let Some(hashes) = raw_hashes {
            if b == b'"' {
                let mut matched: usize = 0;
                let mut scan = idx.saturating_add(1);
                while matched < hashes && bytes.get(scan) == Some(&b'#') {
                    matched = matched.saturating_add(1);
                    scan = scan.saturating_add(1);
                }
                if matched == hashes {
                    raw_hashes = None;
                    idx = scan;
                    continue;
                }
            }
            idx = idx.saturating_add(1);
            continue;
        }

        if let Some(q) = active_quote {
            if !is_raw && b == b'\\' {
                idx = idx.saturating_add(2);
                continue;
            }
            if !is_raw
                && b == b'$'
                && bytes.get(idx.saturating_add(1)) == Some(&b'{')
                && has_matching_interpolation_brace_before_quote(bytes, idx.saturating_add(2), q)
            {
                interp_stack.push((q, is_raw, brace_depth));
                brace_depth = brace_depth.saturating_add(1);
                active_quote = None;
                is_raw = false;
                idx = idx.saturating_add(2);
                continue;
            }
            if b == q {
                active_quote = None;
                is_raw = false;
            }
            idx = idx.saturating_add(1);
            continue;
        }

        if b == b'/' && bytes.get(idx.saturating_add(1)) == Some(&b'/') {
            return line.get(..idx).unwrap_or("");
        }

        if b == b'{' {
            brace_depth = brace_depth.saturating_add(1);
            idx = idx.saturating_add(1);
            continue;
        }

        if b == b'}' {
            if let Some(&(saved_quote, saved_raw, saved_depth)) = interp_stack.last() {
                if brace_depth == saved_depth.saturating_add(1) {
                    interp_stack.pop();
                    brace_depth = saved_depth;
                    active_quote = Some(saved_quote);
                    is_raw = saved_raw;
                    idx = idx.saturating_add(1);
                    continue;
                }
            }
            brace_depth = brace_depth.saturating_sub(1);
            idx = idx.saturating_add(1);
            continue;
        }

        if let Some((hashes, quote_idx)) = rust_raw_string_hash_count(bytes, idx) {
            raw_hashes = Some(hashes);
            idx = quote_idx.saturating_add(1);
            continue;
        }

        if is_valid_raw_string_prefix(bytes, idx, true)
            && (bytes.get(idx.saturating_add(1)) == Some(&b'\'')
                || bytes.get(idx.saturating_add(1)) == Some(&b'"'))
        {
            is_raw = true;
            idx = idx.saturating_add(1);
            continue;
        }

        if b == b'"' {
            active_quote = Some(b'"');
            idx = idx.saturating_add(1);
            continue;
        }

        if b == b'\'' {
            if let Some(char_len) = line.get(idx..).and_then(single_char_literal_byte_len) {
                idx = idx.saturating_add(char_len);
                is_raw = false;
                continue;
            }
            if is_rust_lifetime_at(bytes, idx) {
                idx = idx.saturating_add(1);
                continue;
            }
            if has_unescaped_closing_quote(bytes, idx.saturating_add(1), b'\'') {
                active_quote = Some(b'\'');
            }
            idx = idx.saturating_add(1);
            continue;
        }

        is_raw = false;
        idx = idx.saturating_add(1);
    }

    line
}

/// Strips leading and multi-line Dart annotations while updating `dart_annotation_paren_depth` (`REQ-0605` `AC-02`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives for `strip_dart_annotations_tracking_state` (`REQ-0605` `AC-02`).
///
/// ## 2. Invariants & Mathematical Properties
/// - When `*dart_annotation_paren_depth > 0`, consumes continuation tokens of a multi-line `@annotation(...)` until matching `)` brings depth to `0`.
/// - When encountering a new leading `@identifier(` whose `)` does not close on the same line, records the remaining open parenthesis depth in `*dart_annotation_paren_depth` and returns `""`.
///
/// ## 3. Preconditions
/// - `code` is a comment-stripped UTF-8 line slice; `dart_annotation_paren_depth` tracks open annotation parentheses across lines.
///
/// ## 4. Postconditions
/// - Returns the remaining code slice after all closed annotations on `code`, or `""` while inside an unclosed annotation.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) where N is `code.len()`.
/// - Space Complexity: O(1) zero heap allocation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Non-panicking `.get(..)` slicing and saturating depth counters.
/// - State: Mutates only `*dart_annotation_paren_depth`.
/// - Action: Zero panics, unwraps, or raw slice indexing.
/// - Resource: Zero heap allocations.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0605], [REQ-SOURCE-AST-STRIP-DART-ANNOTATIONS]
pub fn strip_dart_annotations_tracking_state<'a>(
    mut code: &'a str,
    dart_annotation_paren_depth: &mut usize,
) -> &'a str {
    if *dart_annotation_paren_depth > 0 {
        let bytes = code.as_bytes();
        let mut idx: usize = 0;
        let mut active_quote: Option<u8> = None;
        let mut closed_at: Option<usize> = None;
        while let Some(&b) = bytes.get(idx) {
            if let Some(q) = active_quote {
                if b == b'\\' {
                    idx = idx.saturating_add(2);
                    continue;
                }
                if b == q {
                    active_quote = None;
                }
                idx = idx.saturating_add(1);
                continue;
            }
            if b == b'\'' || b == b'"' {
                active_quote = Some(b);
            } else if b == b'(' {
                *dart_annotation_paren_depth = dart_annotation_paren_depth.saturating_add(1);
            } else if b == b')' {
                *dart_annotation_paren_depth = dart_annotation_paren_depth.saturating_sub(1);
                if *dart_annotation_paren_depth == 0 {
                    closed_at = Some(idx.saturating_add(1));
                    break;
                }
            }
            idx = idx.saturating_add(1);
        }
        match closed_at {
            Some(end) => {
                code = code.get(end..).unwrap_or("");
            }
            None => return "",
        }
    }

    loop {
        let trimmed = code.trim_start();
        if !trimmed.starts_with('@') {
            return trimmed;
        }
        let after_at = match trimmed.get(1..) {
            Some(s) => s,
            None => return "",
        };
        let mut idx = 0;
        let bytes = after_at.as_bytes();
        while let Some(&b) = bytes.get(idx) {
            if b.is_ascii_alphanumeric() || b == b'_' || b == b'.' {
                idx = idx.saturating_add(1);
            } else {
                break;
            }
        }
        if idx == 0 {
            return trimmed;
        }
        let rest = match after_at.get(idx..) {
            Some(r) => r,
            None => return "",
        };
        let rest_trimmed = rest.trim_start();
        if rest_trimmed.starts_with('(') {
            let r_bytes = rest_trimmed.as_bytes();
            let mut p_idx: usize = 0;
            let mut paren_depth: usize = 0;
            let mut active_quote: Option<u8> = None;
            let mut paren_end = None;
            while let Some(&b) = r_bytes.get(p_idx) {
                if let Some(q) = active_quote {
                    if b == b'\\' {
                        p_idx = p_idx.saturating_add(2);
                        continue;
                    }
                    if b == q {
                        active_quote = None;
                    }
                    p_idx = p_idx.saturating_add(1);
                    continue;
                }
                if b == b'\'' || b == b'"' {
                    active_quote = Some(b);
                } else if b == b'(' {
                    paren_depth = paren_depth.saturating_add(1);
                } else if b == b')' {
                    paren_depth = paren_depth.saturating_sub(1);
                    if paren_depth == 0 {
                        paren_end = Some(p_idx.saturating_add(1));
                        break;
                    }
                }
                p_idx = p_idx.saturating_add(1);
            }
            if let Some(end) = paren_end {
                code = rest_trimmed.get(end..).unwrap_or("");
                continue;
            } else {
                *dart_annotation_paren_depth = paren_depth;
                return "";
            }
        }
        code = rest;
    }
}

/// Strips leading Dart annotations (e.g. `@override`, `@visibleForTesting`, `@pragma(...)`) from a code line.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A Dart member declaration classification rules.
/// Ensures annotated methods, getters, and constructors are accurately recognized by AST classifiers.
///
/// ## 2. Invariants & Mathematical Properties
/// - Loops until all leading `@identifier` and `@identifier(...)` annotations are stripped.
/// - Skips balanced parentheses inside annotation argument lists.
///
/// ## 3. Preconditions
/// - `code` is a borrowed UTF-8 string slice.
///
/// ## 4. Postconditions
/// - Returns the remaining slice after stripping all leading annotations and whitespace.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(A) where A is the byte length of leading annotations.
/// - Space Complexity: O(1) zero heap allocation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Total function; returns empty string if line contains only annotations.
/// - Never panics on unbalanced parentheses.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SOURCE-AST-STRIP-DART-ANNOTATIONS]
pub fn strip_leading_dart_annotations(code: &str) -> &str {
    let mut depth: usize = 0;
    strip_dart_annotations_tracking_state(code, &mut depth)
}

/// Extracts a Dart getter identifier (`<Type> get <name> => ...` or `get <name> => ...`) from a code line.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A Dart 3 member classification.
/// Allows getters with or without explicit return types and optional annotations to be classified as callables.
///
/// ## 2. Invariants & Mathematical Properties
/// - Requires `" get "` keyword token or leading `"get "` prefix.
/// - Extracts the following alphanumeric identifier before `=>`, `{`, `;`, or whitespace.
///
/// ## 3. Preconditions
/// - `code` is a borrowed UTF-8 string slice.
///
/// ## 4. Postconditions
/// - Returns `Some(identifier)` if the line defines a Dart getter, or `None`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) where N is `code.len()`.
/// - Space Complexity: O(I) for the extracted identifier string.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Non-panicking token scanner.
/// - State: Pure function.
/// - Action: Rejects standard function calls with `get` prefixes (e.g. `getWidget()`).
/// - Resource: At most one string allocation on successful match.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SOURCE-AST-EXTRACT-DART-GETTER]
pub fn extract_dart_getter_ident(code: &str) -> Option<String> {
    let stripped = strip_leading_dart_annotations(code).trim();
    let s = stripped.strip_prefix("static ").unwrap_or(stripped).trim();
    let after_get = if let Some(rest) = s.strip_prefix("get ") {
        rest.trim_start()
    } else if let Some(idx) = s.find(" get ") {
        s.get(idx.saturating_add(5)..)?.trim_start()
    } else if let Some(idx) = s.find("\tget ") {
        s.get(idx.saturating_add(5)..)?.trim_start()
    } else {
        let idx = s.find(" get\t")?;
        s.get(idx.saturating_add(5)..)?.trim_start()
    };

    let mut end: usize = 0;
    let bytes = after_get.as_bytes();
    while let Some(&b) = bytes.get(end) {
        if b.is_ascii_alphanumeric() || b == b'_' {
            end = end.saturating_add(1);
        } else {
            break;
        }
    }
    if end == 0 {
        return None;
    }
    let ident = after_get.get(..end)?;
    let remainder = after_get.get(end..)?.trim_start();
    if remainder.is_empty()
        || remainder.starts_with("=>")
        || remainder.starts_with('{')
        || remainder.starts_with(';')
    {
        Some(ident.to_string())
    } else {
        None
    }
}

/// Updates `brace_depth` and `paren_depth` across a single comment-stripped line during AST extraction.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A delimiter tracking parity (`update_ast_delimiter_depths`).
/// Ensures named parameter braces `({super.key})` inside `(...)` never alter `brace_depth`,
/// and handles nested quotes inside string interpolations `${...}` and raw strings (`r'...'`, `r#"..."#`).
///
/// ## 2. Invariants & Mathematical Properties
/// - `paren_depth` updates on unquoted `'('` and `')'`.
/// - `brace_depth` updates on unquoted `'{'` and `'}'` iff `*paren_depth == 0`.
/// - Tracks interpolation stack `interp_stack` for verified `${...}` expressions inside strings.
///
/// ## 3. Preconditions
/// - `code: &str` is a comment-stripped source line; `brace_depth` and `paren_depth` are mutable counters.
///
/// ## 4. Postconditions
/// - Updates `*brace_depth` and `*paren_depth` in place.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) single-pass byte scan over `code`.
/// - Space Complexity: O(D) where D is max interpolation nesting depth.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Uses `.get(..)` and saturating arithmetic.
/// - State: Mutates only the two depth references.
/// - Action: Skips escaped bytes and hashed raw strings inside string and character literals.
/// - Resource: Bounded stack allocation for interpolation frames.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-UPDATE-AST-DELIMITER-DEPTHS]
pub fn update_ast_delimiter_depths(code: &str, brace_depth: &mut usize, paren_depth: &mut usize) {
    let bytes = code.as_bytes();
    let mut idx: usize = 0;
    let mut active_quote: Option<u8> = None;
    let mut is_raw = false;
    let mut raw_hashes: Option<usize> = None;
    let mut interp_stack: Vec<(u8, usize)> = Vec::new();

    while let Some(&b) = bytes.get(idx) {
        if let Some(hashes) = raw_hashes {
            if b == b'"' {
                let mut matched: usize = 0;
                let mut scan = idx.saturating_add(1);
                while matched < hashes && bytes.get(scan) == Some(&b'#') {
                    matched = matched.saturating_add(1);
                    scan = scan.saturating_add(1);
                }
                if matched == hashes {
                    raw_hashes = None;
                    idx = scan;
                    continue;
                }
            }
            idx = idx.saturating_add(1);
            continue;
        }

        if let Some(q) = active_quote {
            if !is_raw && b == b'\\' {
                idx = idx.saturating_add(2);
                continue;
            }
            if !is_raw
                && b == b'$'
                && bytes.get(idx.saturating_add(1)) == Some(&b'{')
                && has_matching_interpolation_brace_before_quote(bytes, idx.saturating_add(2), q)
            {
                interp_stack.push((q, *brace_depth));
                *brace_depth = brace_depth.saturating_add(1);
                active_quote = None;
                idx = idx.saturating_add(2);
                continue;
            }
            if b == q {
                active_quote = None;
                is_raw = false;
            }
            idx = idx.saturating_add(1);
            continue;
        }

        if b == b'}' {
            if let Some(&(saved_quote, saved_depth)) = interp_stack.last() {
                if *brace_depth == saved_depth.saturating_add(1) {
                    interp_stack.pop();
                    *brace_depth = saved_depth;
                    active_quote = Some(saved_quote);
                    idx = idx.saturating_add(1);
                    continue;
                }
            }
            if *paren_depth == 0 {
                *brace_depth = brace_depth.saturating_sub(1);
            }
            idx = idx.saturating_add(1);
            continue;
        }

        if let Some((hashes, quote_idx)) = rust_raw_string_hash_count(bytes, idx) {
            raw_hashes = Some(hashes);
            idx = quote_idx.saturating_add(1);
            continue;
        }

        if is_valid_raw_string_prefix(bytes, idx, true)
            && (bytes.get(idx.saturating_add(1)) == Some(&b'\'')
                || bytes.get(idx.saturating_add(1)) == Some(&b'"'))
        {
            is_raw = true;
            idx = idx.saturating_add(1);
            continue;
        }

        if b == b'"' {
            active_quote = Some(b'"');
        } else if b == b'\'' {
            if let Some(char_len) = code.get(idx..).and_then(single_char_literal_byte_len) {
                idx = idx.saturating_add(char_len);
                is_raw = false;
                continue;
            }
            if is_rust_lifetime_at(bytes, idx) {
                idx = idx.saturating_add(1);
                continue;
            }
            if has_unescaped_closing_quote(bytes, idx.saturating_add(1), b'\'') {
                active_quote = Some(b'\'');
            }
        } else if b == b'(' {
            *paren_depth = paren_depth.saturating_add(1);
        } else if b == b')' {
            *paren_depth = paren_depth.saturating_sub(1);
        } else if b == b'{' && *paren_depth == 0 {
            *brace_depth = brace_depth.saturating_add(1);
        }

        idx = idx.saturating_add(1);
    }
}

/// Detects direct range-slice indexing expressions (`ident[a..b]`) outside string and char literals.
///
/// # Summary
/// Scans a comment-stripped source line outside string and character literals for `[` ... `..` ... `]`
/// immediately preceded by an identifier or closing delimiter (`alphanumeric`, `_`, `)`, `]`).
///
/// # Preconditions
/// - `code_line` is a valid UTF-8 source line slice after line-comment stripping.
///
/// # Postconditions
/// - Returns `true` if any direct range-slice indexing expression is present outside literals; `false` otherwise.
///
/// # Invariants
/// - Ignores `[` and `..` tokens inside normal string literals, raw string literals, and character literals.
/// - Uses `.get(..)` and saturating arithmetic exclusively without direct slice indexing.
///
/// # Failure Modes
/// - Unclosed literals on a single line terminate scanning safely at `bytes.len()` without panicking.
///
/// # Side Effects
/// - Pure stateless predicate; performs zero I/O and mutates no external state.
///
/// # Determinism
/// - Deterministic O(N) byte scan over `code_line.as_bytes()`.
///
/// # Parameters
/// - `code_line`: Comment-stripped UTF-8 source line slice to inspect.
///
/// # Returns
/// - `bool` (`true` if a direct range-slice indexing expression is detected outside literals).
///
/// # Errors
/// - Infallible predicate; returns `bool` directly without error conditions.
///
/// # Panics
/// - Never panics (`#![forbid(clippy::indexing_slicing)]` compliant via `.get(..)`).
///
/// # Safety
/// - Safe pure lexical byte scan (`#![forbid(unsafe_code)]` compliant).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and Check 34 (`check_zero_regex`) non-panicking slice invariants.
/// Provides general AST-level verification that production code never uses panicking `&s[a..b]` syntax.
///
/// ## 2. Invariants & Mathematical Properties
/// - Matches `[` at byte index `idx > 0` iff `bytes.get(idx - 1)` is in `[A-Za-z0-9_)\]]` and a `..`
///   token appears before the matching `]` at any bracket depth inside the indexing expression.
///
/// ## 3. Preconditions
/// - `code_line: &str` is a source code line.
///
/// ## 4. Postconditions
/// - Returns `true` iff a direct range-slice indexing operation exists on `code_line` outside comments and string/character literals.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) where N is `code_line.len()`.
/// - Space Complexity: O(N) for the lexically sanitized line buffer.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Non-panicking `.get(..)` byte access throughout.
/// - State: Stateless local indices with `RustStringScannerState::Normal`.
/// - Action: Returns `false` on clean lines.
/// - Resource: Bounded single-line buffer allocation.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SOURCE-AST-CHECK34-RANGE-SLICE-DETECTOR]
pub fn contains_direct_range_slice(code_line: &str) -> bool {
    let sanitized =
        scan_rust_line_preserving_lex_state(code_line, &mut RustStringScannerState::Normal);
    let bytes = sanitized.as_bytes();
    let mut idx: usize = 0;
    while let Some(&b) = bytes.get(idx) {
        if b == b'[' {
            let prev_is_indexable = idx > 0
                && bytes.get(idx.saturating_sub(1)).is_some_and(|p| {
                    p.is_ascii_alphanumeric() || *p == b'_' || *p == b')' || *p == b']'
                });
            if prev_is_indexable {
                let mut depth: usize = 1;
                let mut scan = idx.saturating_add(1);
                while let Some(&sb) = bytes.get(scan) {
                    if sb == b'[' {
                        depth = depth.saturating_add(1);
                    } else if sb == b']' {
                        depth = depth.saturating_sub(1);
                        if depth == 0 {
                            break;
                        }
                    } else if sb == b'.' && bytes.get(scan.saturating_add(1)) == Some(&b'.') {
                        return true;
                    }
                    scan = scan.saturating_add(1);
                }
            }
        }
        idx = idx.saturating_add(1);
    }
    false
}

/// Canonical BP-08 C_CST_01 upper ceiling on executable SLOC per Rust callable.
/// Realises: [REQ-0608]
const MAX_CST_FN_EXEC_SLOC: u32 = 50;

/// Canonical BP-08 C_CST_02 upper ceiling on formal parameters excluding self receivers.
/// Realises: [REQ-0608]
const MAX_CST_FN_PARAM_COUNT: u32 = 5;

/// Canonical BP-08 C_CST_06 target compacted docstring line count on RustTierBC items.
/// Realises: [REQ-0608]
const REQUIRED_TIER_BC_DOC_LINES: u32 = 2;

/// Canonical BP-08 compound boolean predicate arity ceiling for pure total guards.
/// Realises: [REQ-0608]
const REQUIRED_MAX_PREDICATE_ARITY: u32 = 3;

/// Normative deterministic UUIDv5 namespace binding for BP-08 / REQ-0608 verification.
/// Realises: [REQ-0608]
const BP08_DETERMINISTIC_UUID: &str = "6ba7b810-9dad-11d1-80b4-00c04fd430c8";

/// Complexity and determinism precondition envelope governing `LosslessRustCstAnalyzer` execution.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C structural
/// complexity and deterministic tool qualification standards (REQ-0608, FLS `fls_izdv9i4spokw`,
/// SC-BP08-05). Encodes formal invariants C_CST_01 through C_CST_06 plus predicate arity, lossless
/// roundtrip fidelity, and UUIDv5 traceability required before entering `ParsingLosslessCst`.
///
/// ## 2. Invariants & Mathematical Properties
/// - C_CST_01 (`max_fn_exec_sloc`): Must satisfy `1 <= max_fn_exec_sloc <= 50`.
/// - C_CST_02 (`max_fn_param_count`): Must satisfy `0 <= max_fn_param_count <= 5`.
/// - C_CST_03 (`max_mut_bool_flags`): Must equal `0`.
/// - C_CST_04 (`stage_fixed_point_mismatch_files`): Must equal `0`.
/// - C_CST_05 (`runtime_llm_token_count`): Must equal `0`.
/// - C_CST_06 (`tier_bc_max_doc_lines`): Must equal `2`.
/// - Total Guard & Identity Invariants: `max_predicate_arity == 3`,
///   `lossless_roundtrip_required == true`, and
///   `deterministic_uuid == "6ba7b810-9dad-11d1-80b4-00c04fd430c8"`.
///
/// ## 3. Preconditions
/// - Constructed via `CstComplexityEnvelope::default()` or explicit field initialization prior to
///   calling `verify_analyzer_envelope_and_seal_receipt`.
///
/// ## 4. Postconditions
/// - Validated fail-closed before binding into an immutable `SupplyChainVerificationReceipt`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(|deterministic_uuid|) comparison and cloning.
/// - Space Complexity: O(|deterministic_uuid|) single owned UUID string plus fixed scalar fields.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Strongly typed scalar `u32`, `bool`, and `String` fields with zero dynamic untyped maps.
/// - State: Value record supporting deterministic ordering (`Ord`, `Eq`) and serialization.
/// - Action: Any drifted threshold triggers `OfflineLockGovernanceError::CstConstraintEnvelopeViolation` (`DEAP-E0803`).
/// - Resource: Bounded fixed-width scalar layout and 36-byte canonical UUID string.
///
/// ## 7. Realization & Traceability
/// Realises: [LosslessRustCstAnalyzer/CstComplexityEnvelope] | [REQ-0608] | SysML v2: part def LosslessRustCstAnalyzer
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct CstComplexityEnvelope {
    /// Upper bound on executable lines of code per Rust function under C_CST_01 (1..=50).
    /// Realises: [REQ-0608]
    pub max_fn_exec_sloc: u32,
    /// Upper bound on formal parameters per Rust callable excluding self receivers under C_CST_02 (0..=5).
    /// Realises: [REQ-0608]
    pub max_fn_param_count: u32,
    /// Permitted count of mutable boolean local flags per function body under C_CST_03 (0).
    /// Realises: [REQ-0608]
    pub max_mut_bool_flags: u32,
    /// Permitted compound boolean predicate arity for pure total guards (3).
    /// Realises: [REQ-0608]
    pub max_predicate_arity: u32,
    /// Permitted Stage-1 versus Stage-2 self-host SHA-256 mismatch file count under C_CST_04 (0).
    /// Realises: [REQ-0608]
    pub stage_fixed_point_mismatch_files: u32,
    /// Permitted runtime LLM token count across CST analysis and codemod passes under C_CST_05 (0).
    /// Realises: [REQ-0608]
    pub runtime_llm_token_count: u32,
    /// Target compacted docstring line count on RustTierBC items under C_CST_06 (2).
    /// Realises: [REQ-0608]
    pub tier_bc_max_doc_lines: u32,
    /// Requires byte-for-byte lossless Rowan green-tree roundtrip preservation (`true`).
    /// Realises: [REQ-0608]
    pub lossless_roundtrip_required: bool,
    /// Canonical UUIDv5 requirement identity (`6ba7b810-9dad-11d1-80b4-00c04fd430c8`).
    /// Realises: [REQ-0608]
    pub deterministic_uuid: String,
}

impl Default for CstComplexityEnvelope {
    /// Constructs the canonical BP-08 C_CST_01..C_CST_06 complexity envelope for REQ-0608.
    /// Realises: [REQ-0608]
    fn default() -> Self {
        Self {
            max_fn_exec_sloc: MAX_CST_FN_EXEC_SLOC,
            max_fn_param_count: MAX_CST_FN_PARAM_COUNT,
            max_mut_bool_flags: 0,
            max_predicate_arity: REQUIRED_MAX_PREDICATE_ARITY,
            stage_fixed_point_mismatch_files: 0,
            runtime_llm_token_count: 0,
            tier_bc_max_doc_lines: REQUIRED_TIER_BC_DOC_LINES,
            lossless_roundtrip_required: true,
            deterministic_uuid: BP08_DETERMINISTIC_UUID.to_string(),
        }
    }
}

/// Cryptographically sealed supply-chain and CST envelope receipt authorizing `Idle -> ParsingLosslessCst`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C air-gapped
/// verification handshake controls (REQ-0608, FLS `fls_izdv9i4spokw`, SC-BP08-02, SC-BP08-04).
/// Binds immutable `Cargo.toml` and `Cargo.lock` SHA-256 digests, `BTreeSet`-ordered verified
/// package coordinates, and a validated `CstComplexityEnvelope` into a SHA-256-sealed token.
///
/// ## 2. Invariants & Mathematical Properties
/// - Ordered Determinism: `verified_crates` and `workspace_members` are stored in `BTreeSet<String>`
///   lexical order with zero hash-seed variance.
/// - Cryptographic Binding: `seal_sha256` commits to `cargo_toml_sha256`, `cargo_lock_sha256`,
///   `BTreeMap`-ordered package checksums, `verified_crates`, `workspace_members`, and `envelope`.
/// - Lifecycle Guard: Valid receipts sealed via `verify_analyzer_envelope_and_seal_receipt` always
///   carry `supply_chain_receipt_sealed == true`.
///
/// ## 3. Preconditions
/// - Produced exclusively after `LockManifestSnapshot`, pinned CST graph verification, and
///   `CstComplexityEnvelope` validation succeed without faults.
///
/// ## 4. Postconditions
/// - Authorizes `LosslessRustCstAnalyzer` transition from `Idle` to `ParsingLosslessCst` and supports
///   post-execution TOCTOU SHA-256 parity verification via `verify_snapshot_parity`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(P log P) construction and parity verification over P locked packages.
/// - Space Complexity: O(P + M) for P verified crate keys and M workspace member identifiers.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Exposes deterministic `[u8; 32]` digests and `BTreeSet<String>` inventories.
/// - State: Value token enabling fail-closed detection of post-seal manifest, lockfile, or seal drift.
/// - Action: Suppresses emission entirely if any supply-chain or C_CST_01..C_CST_06 check fails.
/// - Resource: Bounded by verified workspace `Cargo.lock` package cardinality.
///
/// ## 7. Realization & Traceability
/// Realises: [LosslessRustCstAnalyzer/SupplyChainVerificationReceipt] | [REQ-0608] | SysML v2: part def LosslessRustCstAnalyzer
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct SupplyChainVerificationReceipt {
    /// Immutable 32-byte SHA-256 digest of the verified workspace Cargo.toml snapshot.
    /// Realises: [REQ-0608]
    pub cargo_toml_sha256: [u8; 32],
    /// Immutable 32-byte SHA-256 digest of the verified workspace Cargo.lock snapshot.
    /// Realises: [REQ-0608]
    pub cargo_lock_sha256: [u8; 32],
    /// Lexically ordered `<crate>@<version>` identifiers verified in the locked dependency DAG.
    /// Realises: [REQ-0608]
    pub verified_crates: BTreeSet<String>,
    /// Lexically ordered workspace member crate names covered by the air-gapped verification gate.
    /// Realises: [REQ-0608]
    pub workspace_members: BTreeSet<String>,
    /// Validated C_CST_01..C_CST_06 complexity envelope bound to this verification session.
    /// Realises: [REQ-0608]
    pub envelope: CstComplexityEnvelope,
    /// Deterministic 32-byte SHA-256 cryptographic seal over snapshot digests, packages, and envelope.
    /// Realises: [REQ-0608]
    pub seal_sha256: [u8; 32],
    /// Lifecycle transition guard flag confirming the receipt was sealed (`supplyChainReceiptSealed == true`).
    /// Realises: [REQ-0608]
    pub supply_chain_receipt_sealed: bool,
}

impl SupplyChainVerificationReceipt {
    /// Verifies TOCTOU SHA-256 digest parity and cryptographic seal integrity against a `LockManifestSnapshot`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and DO-330 TQL-4 TOCTOU tamper-detection mandates (REQ-0608,
    /// SC-BP08-02, FM-BP08-002). Detects mid-build mutations to `Cargo.toml`, `Cargo.lock`, or
    /// in-memory receipt fields before permitting CST codemod or baseline completion.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Digest Equality: Succeeds only when `cargo_toml_sha256` and `cargo_lock_sha256` match `snapshot`.
    /// - Cryptographic Seal Parity: Recomputes `compute_supply_chain_receipt_seal` over `snapshot`,
    ///   `self.verified_crates`, and `self.envelope` and requires exact 32-byte equality with `self.seal_sha256`.
    ///
    /// ## 3. Preconditions
    /// - `self` is a candidate `SupplyChainVerificationReceipt` and `snapshot` is the current lock snapshot.
    ///
    /// ## 4. Postconditions
    /// - Returns `Ok(())` iff seal flag, digests, workspace members, envelope bounds, and SHA-256 seal match.
    /// - Returns `Err(OfflineLockGovernanceError)` fail-closed on any drift or envelope tampering.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(P) linear scan over `snapshot.packages` and `self.verified_crates`.
    /// - Space Complexity: O(P) transient hashing buffer freed prior to return.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Pure read-only verification returning typed `OfflineLockGovernanceError`.
    /// - State: Performs zero mutations to `self`, `snapshot`, or filesystem artifacts.
    /// - Action: Rejects unsealed tokens, digest mismatches, or tampered seals with `ManifestToLockDriftDetected`.
    /// - Resource: Single bounded `Vec<u8>` buffer for deterministic SHA-256 recomputation.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [LosslessRustCstAnalyzer/verify_snapshot_parity] | [REQ-0608] | SysML v2: part def LosslessRustCstAnalyzer
    pub fn verify_snapshot_parity(
        &self,
        snapshot: &LockManifestSnapshot,
    ) -> Result<(), OfflineLockGovernanceError> {
        if !self.supply_chain_receipt_sealed {
            return Err(OfflineLockGovernanceError::ManifestToLockDriftDetected);
        }
        if self.cargo_toml_sha256 != snapshot.cargo_toml_sha256
            || self.cargo_lock_sha256 != snapshot.cargo_lock_sha256
        {
            return Err(OfflineLockGovernanceError::ManifestToLockDriftDetected);
        }
        if self.workspace_members != snapshot.workspace_members
            || self.verified_crates.len() != snapshot.packages.len()
        {
            return Err(OfflineLockGovernanceError::ManifestToLockDriftDetected);
        }
        validate_cst_complexity_envelope(&self.envelope)?;
        let expected_seal =
            compute_supply_chain_receipt_seal(snapshot, &self.verified_crates, &self.envelope);
        if self.seal_sha256 != expected_seal {
            return Err(OfflineLockGovernanceError::ManifestToLockDriftDetected);
        }
        Ok(())
    }
}

/// Returns the canonical `BTreeSet` of five SHA-256-pinned CST and Rust formatting crates required by BP-08.
/// Realises: [REQ-0608]
fn required_pinned_cst_crates() -> BTreeSet<String> {
    let names = ["ra_ap_syntax", "rowan", "syn", "quote", "prettyplease"];
    let mut set = BTreeSet::new();
    let mut idx = 0usize;
    while let Some(&name) = names.get(idx) {
        set.insert(name.to_string());
        idx = idx.saturating_add(1);
    }
    set
}

/// Evaluates C_CST_01 through C_CST_06, predicate arity, lossless roundtrip, and UUID invariants on `envelope`.
/// Realises: [REQ-0608]
fn validate_cst_complexity_envelope(
    envelope: &CstComplexityEnvelope,
) -> Result<(), OfflineLockGovernanceError> {
    if envelope.max_fn_exec_sloc == 0
        || envelope.max_fn_exec_sloc > MAX_CST_FN_EXEC_SLOC
        || envelope.max_fn_param_count > MAX_CST_FN_PARAM_COUNT
    {
        return Err(OfflineLockGovernanceError::CstConstraintEnvelopeViolation);
    }
    if envelope.max_mut_bool_flags != 0
        || envelope.stage_fixed_point_mismatch_files != 0
        || envelope.runtime_llm_token_count != 0
    {
        return Err(OfflineLockGovernanceError::CstConstraintEnvelopeViolation);
    }
    if envelope.tier_bc_max_doc_lines != REQUIRED_TIER_BC_DOC_LINES
        || envelope.max_predicate_arity != REQUIRED_MAX_PREDICATE_ARITY
        || !envelope.lossless_roundtrip_required
    {
        return Err(OfflineLockGovernanceError::CstConstraintEnvelopeViolation);
    }
    if envelope.deterministic_uuid != BP08_DETERMINISTIC_UUID {
        return Err(OfflineLockGovernanceError::CstConstraintEnvelopeViolation);
    }
    Ok(())
}

/// Appends length-prefixed UTF-8 bytes to `buf` for unambiguous domain-separated SHA-256 hashing.
/// Realises: [REQ-0608]
fn push_len_prefixed_str(buf: &mut Vec<u8>, value: &str) {
    let len_bytes = (value.len() as u64).to_be_bytes();
    buf.extend_from_slice(&len_bytes);
    buf.extend_from_slice(value.as_bytes());
}

/// Computes the deterministic 32-byte SHA-256 seal across the lock snapshot, verified crates, and CST envelope.
/// Realises: [REQ-0608]
fn compute_supply_chain_receipt_seal(
    snapshot: &LockManifestSnapshot,
    verified_crates: &BTreeSet<String>,
    envelope: &CstComplexityEnvelope,
) -> [u8; 32] {
    let mut buf = Vec::with_capacity(512);
    buf.extend_from_slice(b"DEAP-BP08-SUPPLY-CHAIN-RECEIPT-V1");
    buf.extend_from_slice(&snapshot.cargo_toml_sha256);
    buf.extend_from_slice(&snapshot.cargo_lock_sha256);
    buf.push(u8::from(snapshot.offline_locked_enforced));
    buf.push(u8::from(snapshot.vendor_bundle_supported));
    for (pkg_key, rec) in &snapshot.packages {
        push_len_prefixed_str(&mut buf, pkg_key);
        let checksum_str = rec.checksum.as_deref().unwrap_or_default();
        push_len_prefixed_str(&mut buf, checksum_str);
    }
    for crate_key in verified_crates {
        push_len_prefixed_str(&mut buf, crate_key);
    }
    for member in &snapshot.workspace_members {
        push_len_prefixed_str(&mut buf, member);
    }
    buf.extend_from_slice(&envelope.max_fn_exec_sloc.to_be_bytes());
    buf.extend_from_slice(&envelope.max_fn_param_count.to_be_bytes());
    buf.extend_from_slice(&envelope.max_mut_bool_flags.to_be_bytes());
    buf.extend_from_slice(&envelope.max_predicate_arity.to_be_bytes());
    buf.extend_from_slice(&envelope.stage_fixed_point_mismatch_files.to_be_bytes());
    buf.extend_from_slice(&envelope.runtime_llm_token_count.to_be_bytes());
    buf.extend_from_slice(&envelope.tier_bc_max_doc_lines.to_be_bytes());
    buf.push(u8::from(envelope.lossless_roundtrip_required));
    push_len_prefixed_str(&mut buf, &envelope.deterministic_uuid);
    crate::digest::compute_sha256_bytes(&buf)
}

/// Verifies `C_CST_01`..`C_CST_06` analyzer envelope bounds and seals an immutable `SupplyChainVerificationReceipt`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C air-gapped tool
/// qualification mandates (REQ-0608, FLS `fls_izdv9i4spokw`, SC-BP08-02, SC-BP08-05). Enforces the
/// conjunctive `AdmitCst(M, L, P, C)` precondition gate over the locked dependency snapshot,
/// `BTreeSet`-verified package coordinates, and `LosslessRustCstAnalyzer` complexity envelope before
/// authorizing `Idle -> ParsingLosslessCst`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Supply-Chain & Snapshot Parity: Requires `offline_locked_enforced == true`,
///   `vendor_bundle_supported == true`, pinned `ra_ap_syntax`/`rowan`/`syn`/`quote`/`prettyplease`
///   with zero forbidden semantic crates up to `MAX_AST_DEPTH = 64`, and exact set equality between
///   `verified_crates` and the verified `snapshot.packages` key set.
/// - Complexity Envelope Guard: Enforces C_CST_01 (`1 <= max_fn_exec_sloc <= 50`), C_CST_02
///   (`0 <= max_fn_param_count <= 5`), C_CST_03 (`max_mut_bool_flags == 0`), C_CST_04
///   (`stage_fixed_point_mismatch_files == 0`), C_CST_05 (`runtime_llm_token_count == 0`), C_CST_06
///   (`tier_bc_max_doc_lines == 2`), `max_predicate_arity == 3`, `lossless_roundtrip_required == true`,
///   and `deterministic_uuid == "6ba7b810-9dad-11d1-80b4-00c04fd430c8"`.
/// - Function Complexity Compliance: Executes in `<= 44` executable SLOC with `3` formal parameters,
///   `0` mutable boolean flags, and compound boolean predicate arity `<= 3`.
///
/// ## 3. Preconditions
/// - `snapshot` references a candidate `LockManifestSnapshot`.
/// - `verified_crates` references the `BTreeSet<String>` returned by `verify_pinned_cst_dependency_graph`.
/// - `envelope` references the candidate `CstComplexityEnvelope` configured on `LosslessRustCstAnalyzer`.
///
/// ## 4. Postconditions
/// - Returns `Ok(SupplyChainVerificationReceipt)` with `supply_chain_receipt_sealed == true` and a
///   deterministic 32-byte `seal_sha256` digest when all supply-chain and C_CST_01..C_CST_06 guards hold.
/// - Returns `Err(OfflineLockGovernanceError)` fail-closed (`DEAP-E0804` or `DEAP-E0803`) otherwise.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O((V + E) log V) bounded by iterative `MAX_AST_DEPTH = 64` DAG verification and
///   linear `BTreeMap` / `BTreeSet` SHA-256 sealing over V locked packages and E dependency edges.
/// - Space Complexity: O(V + E) deterministic `BTreeMap` and `BTreeSet` collections with zero recursion.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Pure total function over borrowed references returning typed `OfflineLockGovernanceError`.
/// - State: Zero global or filesystem mutations; suppresses partial receipt construction on any fault.
/// - Action: Short-circuits fail-closed on disabled lock policy, pre-receipt empty/drifted crate sets,
///   forbidden `ra_ap_hir` crates, missing CST crates, depth `> 64`, or C_CST_01..C_CST_06 drift.
/// - Resource: Bounded memory proportional to `snapshot.packages.len()` with zero `.unwrap()` or panics.
///
/// ## 7. Realization & Traceability
/// Realises: [LosslessRustCstAnalyzer/verify_analyzer_envelope_and_seal_receipt] | [REQ-0608] | SysML v2: part def LosslessRustCstAnalyzer
pub fn verify_analyzer_envelope_and_seal_receipt(
    snapshot: &LockManifestSnapshot,
    verified_crates: &BTreeSet<String>,
    envelope: &CstComplexityEnvelope,
) -> Result<SupplyChainVerificationReceipt, OfflineLockGovernanceError> {
    if !snapshot.offline_locked_enforced || !snapshot.vendor_bundle_supported {
        return Err(OfflineLockGovernanceError::LockPolicyDisabled);
    }
    if snapshot.packages.is_empty() {
        return Err(OfflineLockGovernanceError::MalformedTomlOrLockSyntax);
    }
    let required_cst = required_pinned_cst_crates();
    let forbidden_cst = BTreeSet::new();
    let expected_verified =
        verify_pinned_cst_dependency_graph(snapshot, &required_cst, &forbidden_cst, MAX_AST_DEPTH)?;
    if verified_crates.is_empty() || verified_crates != &expected_verified {
        return Err(OfflineLockGovernanceError::ManifestToLockDriftDetected);
    }
    validate_cst_complexity_envelope(envelope)?;
    let seal_sha256 = compute_supply_chain_receipt_seal(snapshot, verified_crates, envelope);
    Ok(SupplyChainVerificationReceipt {
        cargo_toml_sha256: snapshot.cargo_toml_sha256,
        cargo_lock_sha256: snapshot.cargo_lock_sha256,
        verified_crates: verified_crates.clone(),
        workspace_members: snapshot.workspace_members.clone(),
        envelope: envelope.clone(),
        seal_sha256,
        supply_chain_receipt_sealed: true,
    })
}

/// Canonical BP-08 C_CST_01 upper ceiling on `usize` executable SLOC per Rust callable.
/// Realises: [REQ-0608]
const MAX_THRESHOLD_FN_EXEC_SLOC: usize = 50;

/// Canonical BP-08 C_CST_02 upper ceiling on `usize` formal parameters excluding self receivers.
/// Realises: [REQ-0608]
const MAX_THRESHOLD_FN_PARAM_COUNT: usize = 5;

/// Canonical BP-08 C_CST_03 required zero ceiling on mutable boolean local flags per callable.
/// Realises: [REQ-0608]
const REQUIRED_THRESHOLD_MUT_BOOL_FLAGS: usize = 0;

/// Canonical BP-08 required compound boolean predicate clause arity ceiling per expression.
/// Realises: [REQ-0608]
const REQUIRED_THRESHOLD_PREDICATE_ARITY: usize = 3;

/// Immutable structural complexity and lossless CST configuration thresholds for `LosslessRustCstAnalyzer`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C structural
/// complexity verification rules (`CstMetricsThresholds`, US-56 / FEAT-42, REQ-0608, FLS
/// `fls_66m4rnbssgig`). Governs per-callable `v2_metrics` enforcement and bounded `rowan`
/// Red/Green syntax tree traversal in `LosslessRustCstAnalyzer`.
///
/// ## 2. Invariants & Mathematical Properties
/// - C_CST_01 (`max_fn_exec_sloc`): Bounded to `1 <= max_fn_exec_sloc <= 50` executable lines.
/// - C_CST_02 (`max_fn_param_count`): Bounded to `0 <= max_fn_param_count <= 5` non-receiver parameters.
/// - C_CST_03 (`max_mut_bool_flags`): Fixed to `0` mutable `bool` local bindings.
/// - Fixed Structural Ceilings: `max_predicate_arity == 3`, `max_ast_depth == 64` (`MAX_AST_DEPTH`),
///   `lossless_roundtrip_required == true`, and
///   `deterministic_uuid == "6ba7b810-9dad-11d1-80b4-00c04fd430c8"`.
///
/// ## 3. Preconditions
/// - Constructed via `CstMetricsThresholds::default()` or explicit struct literal initialization
///   prior to invoking `validate_metrics_thresholds`.
///
/// ## 4. Postconditions
/// - Validated fail-closed by `validate_metrics_thresholds` before any lossless CST parsing or
///   `v2_metrics` extraction executes.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) field access, copy, and default initialization.
/// - Space Complexity: O(1) fixed-size stack value with zero dynamic heap allocation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Strongly typed `usize`, `bool`, and `&'static str` fields with `Copy` and `Ord` semantics.
/// - State: Immutable configuration value with zero interior mutability.
/// - Action: Out-of-envelope mutations fail closed in `validate_metrics_thresholds` with `DEAP-E0803`.
/// - Resource: Constant stack footprint requiring zero dynamic allocations.
///
/// ## 7. Realization & Traceability
/// Realises: [LosslessRustCstAnalyzer/CstMetricsThresholds] | [REQ-0608] | [FLS: fls_66m4rnbssgig] | SysML v2: part def LosslessRustCstAnalyzer
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub struct CstMetricsThresholds {
    /// Upper bound on non-trivia executable lines per Rust callable under C_CST_01 (`1..=50`).
    /// Realises: [REQ-0608]
    pub max_fn_exec_sloc: usize,
    /// Upper bound on formal parameters excluding `self` receivers under C_CST_02 (`0..=5`).
    /// Realises: [REQ-0608]
    pub max_fn_param_count: usize,
    /// Permitted mutable `bool` local flag count per Rust callable under C_CST_03 (`0`).
    /// Realises: [REQ-0608]
    pub max_mut_bool_flags: usize,
    /// Maximum boolean operator clauses permitted in an unhoisted compound predicate (`3`).
    /// Realises: [REQ-0608]
    pub max_predicate_arity: usize,
    /// Hard iterative Red/Green CST traversal depth ceiling (`MAX_AST_DEPTH = 64`).
    /// Realises: [REQ-0608]
    pub max_ast_depth: usize,
    /// Requires bit-for-bit root `rowan::GreenNode` round-trip equality (`true`).
    /// Realises: [REQ-0608]
    pub lossless_roundtrip_required: bool,
    /// Canonical deterministic UUIDv5 namespace identifier (`"6ba7b810-9dad-11d1-80b4-00c04fd430c8"`).
    /// Realises: [REQ-0608]
    pub deterministic_uuid: &'static str,
}

impl Default for CstMetricsThresholds {
    /// Constructs the canonical BP-08 C_CST_01..C_CST_03 analyzer thresholds for REQ-0608.
    /// Realises: [REQ-0608]
    fn default() -> Self {
        Self {
            max_fn_exec_sloc: MAX_THRESHOLD_FN_EXEC_SLOC,
            max_fn_param_count: MAX_THRESHOLD_FN_PARAM_COUNT,
            max_mut_bool_flags: REQUIRED_THRESHOLD_MUT_BOOL_FLAGS,
            max_predicate_arity: REQUIRED_THRESHOLD_PREDICATE_ARITY,
            max_ast_depth: MAX_AST_DEPTH,
            lossless_roundtrip_required: true,
            deterministic_uuid: BP08_DETERMINISTIC_UUID,
        }
    }
}

/// Typed non-exhaustive domain error taxonomy for `LosslessRustCstAnalyzer` (`DEAP-E0801`..`DEAP-E0804`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C fail-closed
/// diagnostic requirements (`CstMetricsError`, US-56 / FEAT-42, REQ-0608, FLS `fls_9ucqbbd0s2yo`).
/// Eliminates untyped `Result<T, String>` signatures across CST parsing, `DocTier` classification,
/// `v2_metrics` extraction, and single-line `rustc`-compatible NDJSON emission.
///
/// ## 2. Invariants & Mathematical Properties
/// - Exhaustive Code Partition: Every variant maps deterministically onto `DEAP-E0801`,
///   `DEAP-E0802`, `DEAP-E0803`, or `DEAP-E0804` via `CstMetricsError::diagnostic_code`.
/// - Single-Line Zero-ANSI Projection: `Display` formatting emits printable single-line text
///   containing zero embedded `0x0A` newline bytes and zero `0x1B` ANSI escape bytes.
/// - Forward Extensibility: Marked `#[non_exhaustive]` so downstream callers match defensively.
///
/// ## 3. Preconditions
/// - Constructed at the exact point of precondition failure, byte/char-boundary mismatch,
///   UTF-8 or Rust syntax rejection, threshold/metrics violation, or CST depth/arithmetic trap.
///
/// ## 4. Postconditions
/// - Preserves structured UTF-8 byte spans `(byte_start, byte_end)` and canonical `DEAP-E080x`
///   codes for lossless conversion into `CoreError` or NDJSON compiler-message records.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) discriminant inspection and span extraction; O(M) single-line formatting.
/// - Space Complexity: O(M) proportional to owned `detail` or `symbol` payload length.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Non-exhaustive typed enum implementing `std::error::Error`, `Display`, and `Ord`.
/// - State: Self-contained immutable error value with zero shared mutable state.
/// - Action: Sanitizes control characters during `Display` formatting without panicking.
/// - Resource: Bounded by single diagnostic detail or symbol identifier length.
///
/// ## 7. Realization & Traceability
/// Realises: [LosslessRustCstAnalyzer/CstMetricsError] | [REQ-0608] | [FLS: fls_9ucqbbd0s2yo] | SysML v2: part def LosslessRustCstAnalyzer
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub enum CstMetricsError {
    /// Out-of-order lifecycle dispatch or unverified parse state precondition (`DEAP-E0801` / `SC-BP08-06`).
    /// Realises: [REQ-0608]
    UnverifiedParsePrecondition {
        /// Lifecycle state or precondition guard name where verification failed.
        /// Realises: [REQ-0608]
        state: &'static str,
    },
    /// Root `rowan::GreenNode` round-trip mismatch, span collision, or non-char-boundary offset (`DEAP-E0802` / `SC-BP08-08`).
    /// Realises: [REQ-0608]
    LosslessRoundtripOrSpanMismatch {
        /// Inclusive UTF-8 start byte offset of the divergent or invalid span.
        /// Realises: [REQ-0608]
        byte_start: usize,
        /// Exclusive UTF-8 end byte offset of the divergent or invalid span.
        /// Realises: [REQ-0608]
        byte_end: usize,
    },
    /// Invalid UTF-8 byte sequence or lossless `ra_ap_syntax` Rust syntax error (`DEAP-E0803` / `SC-BP08-05`).
    /// Realises: [REQ-0608]
    MalformedUtf8OrRustSyntax {
        /// Inclusive UTF-8 start byte offset of the malformed byte or syntax token.
        /// Realises: [REQ-0608]
        byte_start: usize,
        /// Exclusive UTF-8 end byte offset of the malformed byte or syntax token.
        /// Realises: [REQ-0608]
        byte_end: usize,
        /// Descriptive single-line syntax or UTF-8 decoding diagnostic message.
        /// Realises: [REQ-0608]
        detail: String,
    },
    /// `C_CST_01`..`C_CST_03` threshold drift or callable `v2_metrics` constraint violation (`DEAP-E0803` / `SC-BP08-05`).
    /// Realises: [REQ-0608]
    ThresholdEnvelopeOrMetricsViolation {
        /// Violated constraint rule identifier (such as `"C_CST_01"`, `"C_CST_02"`, or `"C_CST_03"`).
        /// Realises: [REQ-0608]
        code: &'static str,
        /// Offending threshold attribute or callable symbol identifier.
        /// Realises: [REQ-0608]
        symbol: String,
        /// Inclusive UTF-8 start byte offset of the offending callable (`0` for threshold config faults).
        /// Realises: [REQ-0608]
        byte_start: usize,
        /// Exclusive UTF-8 end byte offset of the offending callable (`0` for threshold config faults).
        /// Realises: [REQ-0608]
        byte_end: usize,
    },
    /// Iterative CST traversal ceiling (`MAX_AST_DEPTH = 64`) breach or checked arithmetic overflow (`DEAP-E0804` / `SC-BP08-07`).
    /// Realises: [REQ-0608]
    AstDepthOrArithmeticExhaustion {
        /// Observed CST nesting depth or counter value that triggered the fail-closed halt.
        /// Realises: [REQ-0608]
        observed_depth: usize,
        /// Configured hard traversal or counter ceiling (`MAX_AST_DEPTH = 64`).
        /// Realises: [REQ-0608]
        max_depth: usize,
        /// UTF-8 byte offset of the syntax node or token where traversal halted.
        /// Realises: [REQ-0608]
        byte_offset: usize,
    },
}

impl CstMetricsError {
    /// Returns the canonical `DEAP-E0801`..`DEAP-E0804` diagnostic code for `self`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and DO-330 TQL-4 machine-readable diagnostic classification
    /// (`CstMetricsError::diagnostic_code`, REQ-0608, SC-BP08-05..SC-BP08-08). Supplies the
    /// authoritative `"code"` identifier serialized into `rustc`-compatible NDJSON frames.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Total Partition: Maps `UnverifiedParsePrecondition` to `"DEAP-E0801"`,
    ///   `LosslessRoundtripOrSpanMismatch` to `"DEAP-E0802"`, `MalformedUtf8OrRustSyntax` and
    ///   `ThresholdEnvelopeOrMetricsViolation` to `"DEAP-E0803"`, and
    ///   `AstDepthOrArithmeticExhaustion` to `"DEAP-E0804"`.
    ///
    /// ## 3. Preconditions
    /// - `self` is any initialized `CstMetricsError` variant.
    ///
    /// ## 4. Postconditions
    /// - Returns a non-empty `'static` ASCII diagnostic identifier in `DEAP-E0801`..`DEAP-E0804`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(1) constant discriminant branch.
    /// - Space Complexity: O(1) static string slice with zero heap allocation.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Infallible `const fn` over `&self`.
    /// - State: Pure read-only projection.
    /// - Action: Exhaustive compile-time match with zero panics.
    /// - Resource: Zero dynamic memory usage.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [LosslessRustCstAnalyzer/diagnostic_code] | [REQ-0608] | SysML v2: part def LosslessRustCstAnalyzer
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::UnverifiedParsePrecondition { .. } => "DEAP-E0801",
            Self::LosslessRoundtripOrSpanMismatch { .. } => "DEAP-E0802",
            Self::MalformedUtf8OrRustSyntax { .. }
            | Self::ThresholdEnvelopeOrMetricsViolation { .. } => "DEAP-E0803",
            Self::AstDepthOrArithmeticExhaustion { .. } => "DEAP-E0804",
        }
    }

    /// Returns the half-open UTF-8 byte span `(byte_start, byte_end)` associated with `self`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and DO-330 TQL-4 exact source-location provenance requirements
    /// (`CstMetricsError::byte_span`, REQ-0608, SC-BP08-08). Preserves UTF-8 byte coordinates for
    /// 1-based line/column projection in `UnifiedRustcJsonDiagnosticEmitter`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Returns `(0, 0)` for `UnverifiedParsePrecondition`.
    /// - Returns `(byte_offset, byte_offset)` for point traps in `AstDepthOrArithmeticExhaustion`.
    /// - Returns `(byte_start, byte_end)` verbatim for all span-carrying variants.
    ///
    /// ## 3. Preconditions
    /// - `self` is any initialized `CstMetricsError` variant.
    ///
    /// ## 4. Postconditions
    /// - Returns `(usize, usize)` coordinates without arithmetic wraparound or allocation.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(1) constant field projection.
    /// - Space Complexity: O(1) stack tuple return.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Infallible `const fn` over `&self`.
    /// - State: Pure projection with zero mutation.
    /// - Action: Total match over all variants.
    /// - Resource: Zero heap allocations.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [LosslessRustCstAnalyzer/byte_span] | [REQ-0608] | SysML v2: part def LosslessRustCstAnalyzer
    pub const fn byte_span(&self) -> (usize, usize) {
        match self {
            Self::UnverifiedParsePrecondition { .. } => (0, 0),
            Self::LosslessRoundtripOrSpanMismatch {
                byte_start,
                byte_end,
            }
            | Self::MalformedUtf8OrRustSyntax {
                byte_start,
                byte_end,
                ..
            }
            | Self::ThresholdEnvelopeOrMetricsViolation {
                byte_start,
                byte_end,
                ..
            } => (*byte_start, *byte_end),
            Self::AstDepthOrArithmeticExhaustion { byte_offset, .. } => {
                (*byte_offset, *byte_offset)
            }
        }
    }
}

/// Replaces newline (`\n`, `\r`) and ANSI escape (`\x1b`) characters with spaces for single-line diagnostics.
/// Realises: [REQ-0608]
fn clean_diag_line(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if ch == '\n' || ch == '\r' || ch == '\x1b' {
            out.push(' ');
        } else {
            out.push(ch);
        }
    }
    out
}

impl std::fmt::Display for CstMetricsError {
    /// Formats a single-line zero-ANSI diagnostic message carrying the canonical `DEAP-E080x` code.
    /// Realises: [REQ-0608]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let diag = self.diagnostic_code();
        match self {
            Self::UnverifiedParsePrecondition { state } => write!(
                f,
                "[{diag}] unverified CST parse precondition in state '{}'",
                clean_diag_line(state)
            ),
            Self::LosslessRoundtripOrSpanMismatch {
                byte_start,
                byte_end,
            } => write!(
                f,
                "[{diag}] lossless CST roundtrip or UTF-8 span mismatch at bytes {byte_start}..{byte_end}"
            ),
            Self::MalformedUtf8OrRustSyntax {
                byte_start,
                byte_end,
                detail,
            } => write!(
                f,
                "[{diag}] malformed UTF-8 or Rust syntax at bytes {byte_start}..{byte_end}: {}",
                clean_diag_line(detail)
            ),
            Self::ThresholdEnvelopeOrMetricsViolation {
                code,
                symbol,
                byte_start,
                byte_end,
            } => write!(
                f,
                "[{diag}] CST constraint {} violated by '{}' at bytes {byte_start}..{byte_end}",
                clean_diag_line(code),
                clean_diag_line(symbol)
            ),
            Self::AstDepthOrArithmeticExhaustion {
                observed_depth,
                max_depth,
                byte_offset,
            } => write!(
                f,
                "[{diag}] CST depth or arithmetic exhaustion (observed {observed_depth}, max {max_depth}) at byte {byte_offset}"
            ),
        }
    }
}

impl std::error::Error for CstMetricsError {}

/// Constructs a zero-span `DEAP-E0803` threshold configuration violation error for `symbol`.
/// Realises: [REQ-0608]
fn threshold_violation(code: &'static str, symbol: &'static str) -> CstMetricsError {
    CstMetricsError::ThresholdEnvelopeOrMetricsViolation {
        code,
        symbol: symbol.to_string(),
        byte_start: 0,
        byte_end: 0,
    }
}

/// Validates `CstMetricsThresholds` against `C_CST_01`..`C_CST_03` and `MAX_AST_DEPTH = 64` invariants.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C structural
/// analyzer configuration verification (`validate_metrics_thresholds`, MT-US56-01, US-56 / FEAT-42,
/// REQ-0608, FLS `fls_66m4rnbssgig`, FLS `fls_9ucqbbd0s2yo`). Rejects mutated or out-of-envelope
/// analyzer thresholds fail-closed before entering `ParsingLosslessCst`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Enforces C_CST_01 (`1 <= max_fn_exec_sloc <= 50`), C_CST_02 (`max_fn_param_count <= 5`),
///   C_CST_03 (`max_mut_bool_flags == 0`), `max_predicate_arity == 3`,
///   `max_ast_depth == 64` (`MAX_AST_DEPTH`), `lossless_roundtrip_required == true`, and
///   `deterministic_uuid == "6ba7b810-9dad-11d1-80b4-00c04fd430c8"`.
/// - Complexity Budget Compliance: `1` formal parameter, `<= 45` executable SLOC, `0` mutable
///   boolean flags, and compound boolean predicate arity `<= 2`.
///
/// ## 3. Preconditions
/// - `thresholds` references a candidate `CstMetricsThresholds` configuration record.
///
/// ## 4. Postconditions
/// - Returns `Ok(())` iff every threshold and identity field satisfies the BP-08 envelope.
/// - Returns `Err(CstMetricsError::ThresholdEnvelopeOrMetricsViolation)` (`DEAP-E0803`) on the
///   first violated invariant with zero state mutation.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(|deterministic_uuid|) bounded constant-time scalar and 36-byte UUID checks.
/// - Space Complexity: O(1) on `Ok(())`; single small `symbol: String` allocation on `Err`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Total function taking `1` borrowed `&CstMetricsThresholds` parameter and returning
///   typed `Result<(), CstMetricsError>` (zero `Result<T, String>`).
/// - State: Stateless pure validator with zero side effects.
/// - Action: Short-circuits fail-closed (`DEAP-E0803` / `SC-BP08-05`) on any threshold drift.
/// - Resource: Zero recursion, zero unchecked indexing, and zero panicking operations.
///
/// ## 7. Realization & Traceability
/// Realises: [LosslessRustCstAnalyzer/validate_metrics_thresholds] | [REQ-0608] | [FLS: fls_66m4rnbssgig] | [FLS: fls_9ucqbbd0s2yo] | SysML v2: part def LosslessRustCstAnalyzer
pub fn validate_metrics_thresholds(
    thresholds: &CstMetricsThresholds,
) -> Result<(), CstMetricsError> {
    if thresholds.max_fn_exec_sloc == 0 || thresholds.max_fn_exec_sloc > MAX_THRESHOLD_FN_EXEC_SLOC
    {
        return Err(threshold_violation("C_CST_01", "max_fn_exec_sloc"));
    }
    if thresholds.max_fn_param_count > MAX_THRESHOLD_FN_PARAM_COUNT {
        return Err(threshold_violation("C_CST_02", "max_fn_param_count"));
    }
    if thresholds.max_mut_bool_flags != REQUIRED_THRESHOLD_MUT_BOOL_FLAGS {
        return Err(threshold_violation("C_CST_03", "max_mut_bool_flags"));
    }
    if thresholds.max_predicate_arity != REQUIRED_THRESHOLD_PREDICATE_ARITY {
        return Err(threshold_violation(
            "C_CST_PREDICATE_ARITY",
            "max_predicate_arity",
        ));
    }
    if thresholds.max_ast_depth != MAX_AST_DEPTH {
        return Err(threshold_violation("C_CST_MAX_AST_DEPTH", "max_ast_depth"));
    }
    if !thresholds.lossless_roundtrip_required {
        return Err(threshold_violation(
            "C_CST_LOSSLESS_ROUNDTRIP",
            "lossless_roundtrip_required",
        ));
    }
    if thresholds.deterministic_uuid != BP08_DETERMINISTIC_UUID {
        return Err(threshold_violation(
            "C_CST_DETERMINISTIC_UUID",
            "deterministic_uuid",
        ));
    }
    Ok(())
}

/// Sealed proof-carrying lossless `ra_ap_syntax` and `rowan` Concrete Syntax Tree token (`MT-US56-02`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C structural syntax
/// integrity controls (`VerifiedLosslessCst`, US-56 / FEAT-42, REQ-0608, FLS `fls_exe4zodlwfez`,
/// FLS `fls_142vncdktbin`). Encapsulates a zero-error `ra_ap_syntax::SourceFile` Red tree and
/// immutable `rowan::GreenNode` verified for bit-for-bit UTF-8 round-trip parity and `MAX_AST_DEPTH = 64`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Lossless Byte Identity: Leaf token concatenation over `green_node` equals `source_text.as_bytes()`
///   bit-for-bit, and every node/token `TextRange` boundary satisfies `str::is_char_boundary`.
/// - Bounded Depth Proof: `observed_max_depth <= MAX_AST_DEPTH` (`64`), verified iteratively without
///   call-stack recursion.
/// - Type-Level Sealing: Private fields ensure external callers can obtain an instance exclusively
///   through `parse_verified_lossless_cst` (`lossless_red_green_tree_ready == true`).
///
/// ## 3. Preconditions
/// - Constructed exclusively by `parse_verified_lossless_cst` after `validate_metrics_thresholds`,
///   strict UTF-8 decoding, zero-error `SourceFile::parse`, round-trip check, and depth walk pass.
///
/// ## 4. Postconditions
/// - Authorizes `ParsingLosslessCst -> AnalyzingMetricsAndTiers` consumption by `analyze_cst_v2_metrics`
///   with precomputed monotone `line_starts` and immutable Red/Green CST accessors.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) borrowed projection across all accessors; O(N) cloning over buffer size N.
/// - Space Complexity: O(N) proportional to input UTF-8 source length, CST nodes, and line count.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Read-only accessors return total borrowed slices and scalars without `Option` or panics.
/// - State: Immutable value token once sealed; prevents unverified out-of-order metrics extraction.
/// - Action: Suppresses token creation entirely on `DEAP-E0802`, `DEAP-E0803`, or `DEAP-E0804` faults.
/// - Resource: Bounded by verified `MAX_AST_DEPTH = 64` CST node hierarchy.
///
/// ## 7. Realization & Traceability
/// Realises: [LosslessRustCstAnalyzer/VerifiedLosslessCst] | [REQ-0608] | [FLS: fls_exe4zodlwfez] | [FLS: fls_142vncdktbin] | SysML v2: part def LosslessRustCstAnalyzer
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedLosslessCst {
    /// Canonical relative or logical path of the verified Rust source buffer.
    /// Realises: [REQ-0608]
    source_path: String,
    /// Verified strict UTF-8 Rust source text matching the root GreenNode byte-for-byte.
    /// Realises: [REQ-0608]
    source_text: String,
    /// Lossless immutable Rowan GreenNode backing the parsed Rust Concrete Syntax Tree.
    /// Realises: [REQ-0608]
    green_node: rowan::GreenNode,
    /// Typed lossless ra_ap_syntax SourceFile Red-tree root node.
    /// Realises: [REQ-0608]
    source_file: ra_ap_syntax::SourceFile,
    /// Precomputed 0-based UTF-8 byte offsets of each line start in `source_text`.
    /// Realises: [REQ-0608]
    line_starts: Vec<usize>,
    /// Maximum SyntaxNode nesting depth observed during bounded iterative validation (`<= 64`).
    /// Realises: [REQ-0608]
    observed_max_depth: usize,
    /// Total count of SyntaxNode elements verified during iterative CST traversal.
    /// Realises: [REQ-0608]
    verified_node_count: usize,
    /// Lifecycle transition seal confirming `losslessRedGreenTreeReady` preconditions hold.
    /// Realises: [REQ-0608]
    lossless_red_green_tree_ready: bool,
}

impl VerifiedLosslessCst {
    /// Borrows the canonical source file path bound to this verified lossless CST.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and DO-330 TQL-4 diagnostic provenance tracking
    /// (`VerifiedLosslessCst::source_path`, REQ-0608). Preserves exact file attribution for NDJSON spans.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Returns the exact UTF-8 slice captured from `source_path` at `parse_verified_lossless_cst` entry.
    ///
    /// ## 3. Preconditions
    /// - `self` is a sealed `VerifiedLosslessCst` instance.
    ///
    /// ## 4. Postconditions
    /// - Returns a borrowed `&str` path slice without allocation or mutation.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(1) pointer/length projection.
    /// - Space Complexity: O(1) zero heap allocation.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Infallible total accessor.
    /// - State: Pure read-only borrow.
    /// - Action: Never panics.
    /// - Resource: Zero dynamic memory overhead.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [LosslessRustCstAnalyzer/VerifiedLosslessCst_source_path] | [REQ-0608] | SysML v2: part def LosslessRustCstAnalyzer
    pub fn source_path(&self) -> &str {
        &self.source_path
    }

    /// Borrows the verified strict UTF-8 Rust source text backing this CST.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A lossless text fidelity rules (`VerifiedLosslessCst::source_text`,
    /// REQ-0608, SC-BP08-08). Supplies the authoritative UTF-8 buffer verified equal to `green_node`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - `self.source_text().as_bytes()` is bit-for-bit identical to the concatenated CST leaf tokens.
    ///
    /// ## 3. Preconditions
    /// - `self` is a sealed `VerifiedLosslessCst` instance.
    ///
    /// ## 4. Postconditions
    /// - Returns a borrowed `&str` over the verified source compilation unit.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(1) slice borrow.
    /// - Space Complexity: O(1) zero heap allocation.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Infallible total accessor.
    /// - State: Read-only borrow with zero interior mutation.
    /// - Action: Never panics.
    /// - Resource: Zero allocations.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [LosslessRustCstAnalyzer/VerifiedLosslessCst_source_text] | [REQ-0608] | SysML v2: part def LosslessRustCstAnalyzer
    pub fn source_text(&self) -> &str {
        &self.source_text
    }

    /// Borrows the raw UTF-8 byte slice of the verified Rust compilation unit.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A binary round-trip verification (`VerifiedLosslessCst::source_bytes`,
    /// REQ-0608, FLS `fls_exe4zodlwfez`). Enables direct byte-level span slicing via `.get(..)`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Equal in length and byte content to the `source_bytes` accepted by `parse_verified_lossless_cst`.
    ///
    /// ## 3. Preconditions
    /// - `self` is a sealed `VerifiedLosslessCst` instance.
    ///
    /// ## 4. Postconditions
    /// - Returns a borrowed `&[u8]` view of `self.source_text`.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(1) byte slice projection.
    /// - Space Complexity: O(1) zero heap allocation.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Infallible total accessor.
    /// - State: Pure read-only projection.
    /// - Action: Never panics.
    /// - Resource: Zero allocations.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [LosslessRustCstAnalyzer/VerifiedLosslessCst_source_bytes] | [REQ-0608] | SysML v2: part def LosslessRustCstAnalyzer
    pub fn source_bytes(&self) -> &[u8] {
        self.source_text.as_bytes()
    }

    /// Borrows the verified root `rowan::GreenNode` of the lossless Concrete Syntax Tree.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and DO-330 TQL-4 immutable syntax tree persistence
    /// (`VerifiedLosslessCst::green_node`, REQ-0608, SC-BP08-08). Exposes the trivia-preserving
    /// Rowan green tree verified for exact byte parity against `source_bytes`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - `u32::from(self.green_node().text_len()) as usize == self.source_bytes().len()`.
    ///
    /// ## 3. Preconditions
    /// - `self` is a sealed `VerifiedLosslessCst` instance.
    ///
    /// ## 4. Postconditions
    /// - Returns an immutable `&rowan::GreenNode` reference.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(1) constant reference projection.
    /// - Space Complexity: O(1) zero allocation.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Infallible `const fn` over `&self`.
    /// - State: Immutable reference to `Arc`-backed Rowan green data.
    /// - Action: Never panics.
    /// - Resource: Zero heap overhead.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [LosslessRustCstAnalyzer/VerifiedLosslessCst_green_node] | [REQ-0608] | SysML v2: part def LosslessRustCstAnalyzer
    pub const fn green_node(&self) -> &rowan::GreenNode {
        &self.green_node
    }

    /// Borrows the typed `ra_ap_syntax::SourceFile` AST root wrapping the verified Red tree.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A typed syntax inspection (`VerifiedLosslessCst::source_file`,
    /// REQ-0608, FLS `fls_142vncdktbin`). Provides zero-error typed item traversal for `DocTier`
    /// classification and `v2_metrics` extraction.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Guaranteed free of `ra_ap_syntax` parse errors (`errors().is_empty() == true`).
    ///
    /// ## 3. Preconditions
    /// - `self` is a sealed `VerifiedLosslessCst` instance.
    ///
    /// ## 4. Postconditions
    /// - Returns a borrowed `&ra_ap_syntax::SourceFile` root node.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(1) constant field borrow.
    /// - Space Complexity: O(1) zero allocation.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Infallible `const fn` over `&self`.
    /// - State: Read-only borrow.
    /// - Action: Never panics.
    /// - Resource: Zero allocations.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [LosslessRustCstAnalyzer/VerifiedLosslessCst_source_file] | [REQ-0608] | SysML v2: part def LosslessRustCstAnalyzer
    pub const fn source_file(&self) -> &ra_ap_syntax::SourceFile {
        &self.source_file
    }

    /// Borrows the untyped root `ra_ap_syntax::SyntaxNode` of the verified Red syntax tree.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A bounded Red-tree worklist traversal (`VerifiedLosslessCst::syntax_node`,
    /// REQ-0608, SC-BP08-07). Supplies the root `SyntaxNode` verified not to exceed `MAX_AST_DEPTH = 64`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - `self.syntax_node().text_range()` spans `0..self.source_bytes().len()`.
    ///
    /// ## 3. Preconditions
    /// - `self` is a sealed `VerifiedLosslessCst` instance.
    ///
    /// ## 4. Postconditions
    /// - Returns a borrowed `&ra_ap_syntax::SyntaxNode` root reference.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(1) trait projection via `ra_ap_syntax::AstNode::syntax`.
    /// - Space Complexity: O(1) zero allocation.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Infallible total accessor.
    /// - State: Pure read-only borrow.
    /// - Action: Never panics.
    /// - Resource: Zero allocations.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [LosslessRustCstAnalyzer/VerifiedLosslessCst_syntax_node] | [REQ-0608] | SysML v2: part def LosslessRustCstAnalyzer
    pub fn syntax_node(&self) -> &ra_ap_syntax::SyntaxNode {
        ra_ap_syntax::AstNode::syntax(&self.source_file)
    }

    /// Borrows the precomputed monotone UTF-8 byte offsets of each 0-based line start.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A deterministic SLOC and span coordinate resolution
    /// (`VerifiedLosslessCst::line_starts`, REQ-0608). Enables `O(log L)` `binary_search` line lookup
    /// via bounds-safe `.get(..)` indexing.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Strictly increasing sequence starting with `0` (`self.line_starts().first() == Some(&0)`).
    ///
    /// ## 3. Preconditions
    /// - `self` is a sealed `VerifiedLosslessCst` instance.
    ///
    /// ## 4. Postconditions
    /// - Returns a non-empty `&[usize]` slice of line-start byte offsets.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(1) slice projection.
    /// - Space Complexity: O(1) zero allocation.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Infallible total accessor.
    /// - State: Pure read-only slice borrow.
    /// - Action: Never panics.
    /// - Resource: Zero allocations.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [LosslessRustCstAnalyzer/VerifiedLosslessCst_line_starts] | [REQ-0608] | SysML v2: part def LosslessRustCstAnalyzer
    pub fn line_starts(&self) -> &[usize] {
        &self.line_starts
    }

    /// Returns the maximum `SyntaxNode` nesting depth observed during iterative CST validation.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A bounded resource verification (`VerifiedLosslessCst::observed_max_depth`,
    /// REQ-0608, SC-BP08-07, FM-BP08-007). Exposes empirical proof that CST depth satisfies `<= 64`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Guaranteed `0 <= self.observed_max_depth() <= MAX_AST_DEPTH` (`64`), with root `SOURCE_FILE` at `0`.
    ///
    /// ## 3. Preconditions
    /// - `self` is a sealed `VerifiedLosslessCst` instance.
    ///
    /// ## 4. Postconditions
    /// - Returns the exact maximum `SyntaxNode` edge depth encountered across the Red tree.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(1) scalar copy.
    /// - Space Complexity: O(1) stack scalar.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Infallible `const fn`.
    /// - State: Pure projection.
    /// - Action: Never panics.
    /// - Resource: Zero allocations.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [LosslessRustCstAnalyzer/VerifiedLosslessCst_observed_max_depth] | [REQ-0608] | SysML v2: part def LosslessRustCstAnalyzer
    pub const fn observed_max_depth(&self) -> usize {
        self.observed_max_depth
    }

    /// Returns the total count of `SyntaxNode` elements visited during iterative CST validation.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A structural completeness auditing
    /// (`VerifiedLosslessCst::verified_node_count`, REQ-0608, SC-BP08-07). Confirms every interior CST
    /// node was traversed with checked arithmetic.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - `self.verified_node_count() >= 1` because root `SOURCE_FILE` is always visited.
    ///
    /// ## 3. Preconditions
    /// - `self` is a sealed `VerifiedLosslessCst` instance.
    ///
    /// ## 4. Postconditions
    /// - Returns the checked `usize` count of `SyntaxNode` elements in the Red tree.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(1) scalar copy.
    /// - Space Complexity: O(1) stack scalar.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Infallible `const fn`.
    /// - State: Pure projection.
    /// - Action: Never panics.
    /// - Resource: Zero allocations.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [LosslessRustCstAnalyzer/VerifiedLosslessCst_verified_node_count] | [REQ-0608] | SysML v2: part def LosslessRustCstAnalyzer
    pub const fn verified_node_count(&self) -> usize {
        self.verified_node_count
    }

    /// Returns `true` iff the `losslessRedGreenTreeReady` lifecycle precondition seal is active.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A state-machine transition gating
    /// (`VerifiedLosslessCst::is_lossless_red_green_tree_ready`, REQ-0608, SC-BP08-06, FM-BP08-006).
    /// Guards entry into `AnalyzingMetricsAndTiers`.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Evaluates to `true` for every token sealed by `parse_verified_lossless_cst`.
    ///
    /// ## 3. Preconditions
    /// - `self` is a `VerifiedLosslessCst` instance.
    ///
    /// ## 4. Postconditions
    /// - Returns the boolean lifecycle seal state without side effects.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(1) boolean read.
    /// - Space Complexity: O(1) stack scalar.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Infallible `const fn`.
    /// - State: Pure read-only guard check.
    /// - Action: Never panics.
    /// - Resource: Zero allocations.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [LosslessRustCstAnalyzer/is_lossless_red_green_tree_ready] | [REQ-0608] | SysML v2: part def LosslessRustCstAnalyzer
    pub const fn is_lossless_red_green_tree_ready(&self) -> bool {
        self.lossless_red_green_tree_ready
    }
}

/// Decodes `source_bytes` as strict UTF-8 or fails closed with `DEAP-E0803` byte coordinates.
/// Realises: [REQ-0608]
fn decode_strict_utf8_source(source_bytes: &[u8]) -> Result<&str, CstMetricsError> {
    std::str::from_utf8(source_bytes).map_err(|utf8_err| {
        let byte_start = utf8_err.valid_up_to();
        let byte_end = match utf8_err.error_len() {
            Some(err_len) => byte_start.saturating_add(err_len),
            None => source_bytes.len(),
        };
        CstMetricsError::MalformedUtf8OrRustSyntax {
            byte_start,
            byte_end,
            detail: utf8_err.to_string(),
        }
    })
}

/// Verifies `[byte_start, byte_end)` is an ordered, in-bounds UTF-8 character boundary span.
/// Realises: [REQ-0608]
fn verify_utf8_char_boundary_span(
    source_text: &str,
    byte_start: usize,
    byte_end: usize,
) -> Result<(), CstMetricsError> {
    if byte_start > byte_end || byte_end > source_text.len() {
        return Err(CstMetricsError::LosslessRoundtripOrSpanMismatch {
            byte_start,
            byte_end,
        });
    }
    if !source_text.is_char_boundary(byte_start) || !source_text.is_char_boundary(byte_end) {
        return Err(CstMetricsError::LosslessRoundtripOrSpanMismatch {
            byte_start,
            byte_end,
        });
    }
    Ok(())
}

/// Constructs a typed `DEAP-E0804` CST depth or checked-arithmetic exhaustion error.
/// Realises: [REQ-0608]
const fn depth_exhaustion_err(
    observed_depth: usize,
    max_depth: usize,
    byte_offset: usize,
) -> CstMetricsError {
    CstMetricsError::AstDepthOrArithmeticExhaustion {
        observed_depth,
        max_depth,
        byte_offset,
    }
}

/// Increments `current_depth` via `checked_add(1)` and enforces `next_depth <= max_depth` (`DEAP-E0804`).
/// Realises: [REQ-0608]
fn checked_next_cst_depth(
    current_depth: usize,
    max_depth: usize,
    byte_offset: usize,
) -> Result<usize, CstMetricsError> {
    let next_depth = current_depth
        .checked_add(1)
        .ok_or_else(|| depth_exhaustion_err(usize::MAX, max_depth, byte_offset))?;
    if next_depth > max_depth {
        return Err(depth_exhaustion_err(next_depth, max_depth, byte_offset));
    }
    Ok(next_depth)
}

/// Verifies contiguous UTF-8 byte parity and character boundaries for a single CST leaf token.
/// Realises: [REQ-0608]
fn verify_cst_leaf_token_parity(
    token: &ra_ap_syntax::SyntaxToken,
    source_text: &str,
    cursor: usize,
    max_depth: usize,
) -> Result<usize, CstMetricsError> {
    let range = token.text_range();
    let byte_start = u32::from(range.start()) as usize;
    let byte_end = u32::from(range.end()) as usize;
    verify_utf8_char_boundary_span(source_text, byte_start, byte_end)?;
    let token_bytes = token.text().as_bytes();
    let next_cursor = cursor
        .checked_add(token_bytes.len())
        .ok_or_else(|| depth_exhaustion_err(usize::MAX, max_depth, cursor))?;
    if byte_start != cursor || byte_end != next_cursor {
        return Err(CstMetricsError::LosslessRoundtripOrSpanMismatch {
            byte_start,
            byte_end,
        });
    }
    if source_text.as_bytes().get(byte_start..byte_end) != Some(token_bytes) {
        return Err(CstMetricsError::LosslessRoundtripOrSpanMismatch {
            byte_start,
            byte_end,
        });
    }
    Ok(next_cursor)
}

/// Pushes `node` children in reverse lexical order onto `stack` with checked `SyntaxNode` depths.
/// Realises: [REQ-0608]
fn push_cst_children_reversed(
    node: &ra_ap_syntax::SyntaxNode,
    depth: usize,
    max_depth: usize,
    stack: &mut Vec<(ra_ap_syntax::SyntaxElement, usize)>,
) -> Result<(), CstMetricsError> {
    let children: Vec<ra_ap_syntax::SyntaxElement> = node.children_with_tokens().collect();
    for child in children.into_iter().rev() {
        let child_depth = match &child {
            ra_ap_syntax::SyntaxElement::Node(child_node) => {
                let offset = u32::from(child_node.text_range().start()) as usize;
                checked_next_cst_depth(depth, max_depth, offset)?
            }
            ra_ap_syntax::SyntaxElement::Token(_) => depth,
        };
        stack.push((child, child_depth));
    }
    Ok(())
}

/// Iteratively verifies lossless Rowan Green/Red byte parity, UTF-8 boundaries, and `MAX_AST_DEPTH`.
/// Realises: [REQ-0608]
fn verify_green_tree_roundtrip_and_depth(
    green_node: &rowan::GreenNode,
    source_text: &str,
    max_depth: usize,
) -> Result<(usize, usize), CstMetricsError> {
    let root_node = ra_ap_syntax::SyntaxNode::new_root(green_node.clone());
    let mut stack = vec![(ra_ap_syntax::SyntaxElement::Node(root_node), 0usize)];
    let (mut cursor, mut max_seen, mut node_count) = (0usize, 0usize, 0usize);
    while let Some((element, depth)) = stack.pop() {
        match element {
            ra_ap_syntax::SyntaxElement::Node(node) => {
                let range = node.text_range();
                let start = u32::from(range.start()) as usize;
                let end = u32::from(range.end()) as usize;
                if depth > max_depth {
                    return Err(depth_exhaustion_err(depth, max_depth, start));
                }
                verify_utf8_char_boundary_span(source_text, start, end)?;
                max_seen = max_seen.max(depth);
                node_count = node_count
                    .checked_add(1)
                    .ok_or_else(|| depth_exhaustion_err(usize::MAX, max_depth, start))?;
                push_cst_children_reversed(&node, depth, max_depth, &mut stack)?;
            }
            ra_ap_syntax::SyntaxElement::Token(token) => {
                cursor = verify_cst_leaf_token_parity(&token, source_text, cursor, max_depth)?;
            }
        }
    }
    let green_len = u32::from(green_node.text_len()) as usize;
    if cursor != source_text.len() || green_len != source_text.len() {
        return Err(CstMetricsError::LosslessRoundtripOrSpanMismatch {
            byte_start: cursor.min(green_len),
            byte_end: source_text.len().max(green_len),
        });
    }
    Ok((max_seen, node_count))
}

/// Builds the sorted 0-based UTF-8 byte offset table of line starts in `source_text`.
/// Realises: [REQ-0608]
fn compute_monotone_line_starts(source_text: &str) -> Result<Vec<usize>, CstMetricsError> {
    let mut starts = vec![0usize];
    for (idx, byte) in source_text.as_bytes().iter().copied().enumerate() {
        if byte == b'\n' {
            let next_start = idx
                .checked_add(1)
                .ok_or_else(|| depth_exhaustion_err(usize::MAX, MAX_AST_DEPTH, idx))?;
            starts.push(next_start);
        }
    }
    Ok(starts)
}

/// Parses `source_bytes` into a sealed `VerifiedLosslessCst` with `rowan::GreenNode` byte parity and `MAX_AST_DEPTH = 64` guard.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C lossless syntax
/// analysis mandates (`parse_verified_lossless_cst`, MT-US56-02 / MT-US52-02, US-56 / FEAT-42,
/// REQ-0608, FLS `fls_exe4zodlwfez`, FLS `fls_142vncdktbin`). Enforces the `ParsingLosslessCst`
/// entry and exit invariants before sealing `VerifiedLosslessCst` (`losslessRedGreenTreeReady`).
///
/// ## 2. Invariants & Mathematical Properties
/// - Threshold Precondition: Validates `thresholds` via `validate_metrics_thresholds` (`C_CST_01`..`C_CST_03`,
///   `MAX_AST_DEPTH = 64`, `lossless_roundtrip_required == true`, canonical `deterministic_uuid`).
/// - Strict UTF-8 & Zero Syntax Faults: Rejects invalid UTF-8 bytes and non-empty `SourceFile::parse`
///   syntax diagnostics fail-closed (`DEAP-E0803`, `FM-BP08-005`).
/// - Lossless Round-Trip & Depth Ceiling: Iteratively verifies contiguous leaf-token byte equality
///   against `source_bytes`, UTF-8 `str::is_char_boundary` node/token spans (`DEAP-E0802`, `FM-BP08-008`),
///   and `SyntaxNode` nesting depth `<= 64` via `checked_add(1)` (`DEAP-E0804`, `FM-BP08-007`).
/// - Complexity Budget Compliance: `3` formal parameters, `<= 48` executable SLOC, `0` mutable `bool`
///   flags, and compound boolean predicate arity `<= 2`.
///
/// ## 3. Preconditions
/// - `source_path` identifies the logical or workspace-relative `.rs` path for diagnostic spans.
/// - `source_bytes` holds the candidate raw compilation unit byte slice.
/// - `thresholds` references the `CstMetricsThresholds` configuration under verification.
///
/// ## 4. Postconditions
/// - Returns `Ok(VerifiedLosslessCst)` with `is_lossless_red_green_tree_ready() == true` iff all
///   threshold, UTF-8, syntax, byte round-trip, char-boundary, and `MAX_AST_DEPTH = 64` guards pass.
/// - Returns `Err(CstMetricsError)` (`DEAP-E0802`, `DEAP-E0803`, or `DEAP-E0804`) fail-closed otherwise.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) linear in `source_bytes.len()` and CST node/token count with zero recursion.
/// - Space Complexity: O(N) for the lossless Rowan Green/Red CST, explicit traversal stack, and line table.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Total function taking `3` borrowed parameters and returning typed `Result<VerifiedLosslessCst, CstMetricsError>`.
/// - State: Pure stateless constructor performing zero filesystem or global state mutations.
/// - Action: Halts on first threshold, UTF-8, syntax, round-trip, or depth fault with exact byte spans.
/// - Resource: Explicit heap worklist bounded by `MAX_AST_DEPTH = 64` prevents call-stack overflow.
///
/// ## 7. Realization & Traceability
/// Realises: [LosslessRustCstAnalyzer/parse_verified_lossless_cst] | [REQ-0608] | [FLS: fls_exe4zodlwfez] | [FLS: fls_142vncdktbin] | SysML v2: part def LosslessRustCstAnalyzer
pub fn parse_verified_lossless_cst(
    source_path: &str,
    source_bytes: &[u8],
    thresholds: &CstMetricsThresholds,
) -> Result<VerifiedLosslessCst, CstMetricsError> {
    validate_metrics_thresholds(thresholds)?;
    let source_text = decode_strict_utf8_source(source_bytes)?;
    let parse = ra_ap_syntax::SourceFile::parse(source_text, ra_ap_syntax::Edition::CURRENT);
    if let Some(first_error) = parse.errors().first() {
        let range = first_error.range();
        return Err(CstMetricsError::MalformedUtf8OrRustSyntax {
            byte_start: u32::from(range.start()) as usize,
            byte_end: u32::from(range.end()) as usize,
            detail: first_error.to_string(),
        });
    }
    let syntax_node = parse.syntax_node();
    let green_node: rowan::GreenNode = syntax_node.green().to_owned();
    let (observed_max_depth, verified_node_count) =
        verify_green_tree_roundtrip_and_depth(&green_node, source_text, thresholds.max_ast_depth)?;
    let line_starts = compute_monotone_line_starts(source_text)?;
    Ok(VerifiedLosslessCst {
        source_path: source_path.to_string(),
        source_text: source_text.to_string(),
        green_node,
        source_file: parse.tree(),
        line_starts,
        observed_max_depth,
        verified_node_count,
        lossless_red_green_tree_ready: true,
    })
}

/// Documentation and safety-audit tier assigned to a Rust Concrete Syntax Tree item (`MT-US56-03`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C structural
/// documentation tier governance (`DocTier`, US-56 / FEAT-42, REQ-0608, Check 36, FLS
/// `fls_ikfvbeewame7`). Partitions Rust CST declarations into 7-section `RustTierA` contracts,
/// 2-line `RustTierBC` compacted headers (`C_CST_06`), and pruned `TestExcluded` subtrees.
///
/// ## 2. Invariants & Mathematical Properties
/// - Disjoint Exhaustive Partition: Every classified Rust declaration belongs to exactly one of
///   `{RustTierA, RustTierBC, TestExcluded}`.
/// - Production Projection: `self.is_production()` holds iff `self != DocTier::TestExcluded`.
///
/// ## 3. Preconditions
/// - Assigned by `classify_cfg_test_excluded_doc_tier` during bounded iterative Red-tree traversal.
///
/// ## 4. Postconditions
/// - Drives Check-36 docstring synthesis/verification and excludes test subtrees from production SLOC.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) discriminant comparison and label lookup.
/// - Space Complexity: O(1) single-byte copy representation with zero heap allocation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Closed value enum implementing `Copy`, `Ord`, `Eq`, `Hash`, and `Serialize`.
/// - State: Immutable scalar discriminant.
/// - Action: Exhaustive pattern matching prevents unhandled tier states.
/// - Resource: Zero dynamic memory overhead.
///
/// ## 7. Realization & Traceability
/// Realises: [LosslessRustCstAnalyzer/DocTier] | [REQ-0608] | [FLS: fls_ikfvbeewame7] | SysML v2: part def LosslessRustCstAnalyzer
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum DocTier {
    /// Public (`pub`) or generated self-hosted Rust items governed by 7-section DO-178C docstrings.
    /// Realises: [REQ-0608]
    RustTierA,
    /// Internal (`pub(crate)`, `pub(super)`, `pub(in ...)`) or private non-test items (`C_CST_06`).
    /// Realises: [REQ-0608]
    RustTierBC,
    /// Test-scoped (`#[cfg(test)]` subtree or `#[test]` callable) items pruned from production budgets.
    /// Realises: [REQ-0608]
    TestExcluded,
}

impl DocTier {
    /// Returns `true` iff `self` belongs to a non-test production documentation tier.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A production boundary isolation (`DocTier::is_production`,
    /// REQ-0608, SC-BP08-06). Separates production-audited declarations from pruned test items.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Evaluates to `true` for `DocTier::RustTierA` and `DocTier::RustTierBC`, and `false` for
    ///   `DocTier::TestExcluded`.
    ///
    /// ## 3. Preconditions
    /// - `self` is any initialized `DocTier` discriminant.
    ///
    /// ## 4. Postconditions
    /// - Returns a deterministic boolean classification without side effects.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(1) constant discriminant check.
    /// - Space Complexity: O(1) zero heap allocation.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Infallible `const fn`.
    /// - State: Pure predicate.
    /// - Action: Never panics.
    /// - Resource: O(1) stack usage.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [LosslessRustCstAnalyzer/DocTier_is_production] | [REQ-0608] | SysML v2: part def LosslessRustCstAnalyzer
    pub const fn is_production(self) -> bool {
        matches!(self, Self::RustTierA | Self::RustTierBC)
    }

    /// Returns the canonical ASCII identifier string for `self`.
    ///
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A deterministic diagnostic labeling (`DocTier::label`, REQ-0608).
    /// Supplies stable tier names for structured CST inventories and audit reports.
    ///
    /// ## 2. Invariants & Mathematical Properties
    /// - Bijective mapping onto `{"RustTierA", "RustTierBC", "TestExcluded"}`.
    ///
    /// ## 3. Preconditions
    /// - `self` is any initialized `DocTier` discriminant.
    ///
    /// ## 4. Postconditions
    /// - Returns a non-empty `'static` ASCII slice.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(1) constant branch projection.
    /// - Space Complexity: O(1) static memory.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Interface: Infallible `const fn`.
    /// - State: Pure read-only projection.
    /// - Action: Exhaustive match with zero panics.
    /// - Resource: Zero heap allocations.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [LosslessRustCstAnalyzer/DocTier_label] | [REQ-0608] | SysML v2: part def LosslessRustCstAnalyzer
    pub const fn label(self) -> &'static str {
        match self {
            Self::RustTierA => "RustTierA",
            Self::RustTierBC => "RustTierBC",
            Self::TestExcluded => "TestExcluded",
        }
    }
}

/// Verified production `DocTier` descriptor for a single non-test Rust CST declaration (`MT-US56-03`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C structural
/// traceability controls (`ClassifiedDocTierItem`, US-56 / FEAT-42, REQ-0608, SC-BP08-06,
/// SC-BP08-08). Binds an extracted Rust symbol to its verified UTF-8 `TextRange` byte span,
/// production `DocTier`, outer docblock line count, and callable classification.
///
/// ## 2. Invariants & Mathematical Properties
/// - Production-Only Tiering: `self.tier.is_production() == true` (`RustTierA` or `RustTierBC`).
/// - Verified Span Bounds: `byte_start <= byte_end` and both offsets satisfy `str::is_char_boundary`
///   against the owning `VerifiedLosslessCst` source buffer.
///
/// ## 3. Preconditions
/// - Constructed exclusively by `classify_cfg_test_excluded_doc_tier` outside `#[cfg(test)]` and
///   `#[test]` subtrees after UTF-8 character-boundary verification succeeds.
///
/// ## 4. Postconditions
/// - Stored inside `DocTierInventory::production_items` keyed by `(byte_start as u32, symbol)`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) scalar field access; O(|symbol|) cloning.
/// - Space Complexity: O(|symbol|) single owned identifier string plus fixed scalar fields.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Strongly typed value struct implementing `Ord`, `Eq`, and `Serialize`.
/// - State: Immutable value record once inserted into `DocTierInventory`.
/// - Action: Zero panicking operations.
/// - Resource: Bounded by single Rust identifier length.
///
/// ## 7. Realization & Traceability
/// Realises: [LosslessRustCstAnalyzer/ClassifiedDocTierItem] | [REQ-0608] | [FLS: fls_ikfvbeewame7] | SysML v2: part def LosslessRustCstAnalyzer
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct ClassifiedDocTierItem {
    /// Unqualified Rust declaration identifier extracted from the lossless Red CST.
    /// Realises: [REQ-0608]
    pub symbol: String,
    /// Assigned production documentation tier (`DocTier::RustTierA` or `DocTier::RustTierBC`).
    /// Realises: [REQ-0608]
    pub tier: DocTier,
    /// Inclusive UTF-8 start byte offset of the item `SyntaxNode` (`TextRange::start()`).
    /// Realises: [REQ-0608]
    pub byte_start: usize,
    /// Exclusive UTF-8 end byte offset of the item `SyntaxNode` (`TextRange::end()`).
    /// Realises: [REQ-0608]
    pub byte_end: usize,
    /// Verified count of leading outer `///` / `/** */` doc lines and outer `#[doc = ...]` attributes.
    /// Realises: [REQ-0608]
    pub doc_line_count: usize,
    /// True when the classified declaration is an executable `ast::Fn` callable.
    /// Realises: [REQ-0608]
    pub is_callable: bool,
}

/// Deterministic `BTreeMap`/`BTreeSet`-ordered `DocTier` and `cfg(test)`-pruned SLOC inventory (`MT-US56-03`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C deterministic
/// syntax analysis mandates (`DocTierInventory`, US-56 / FEAT-42, REQ-0608, FLS `fls_ikfvbeewame7`,
/// FLS `fls_nsvzzbldhq53`). Captures the `BTreeOrderedItemInventory` and `cfg(test)`-excluded
/// `production_exec_sloc` line set produced in the `AnalyzingMetricsAndTiers` lifecycle state.
///
/// ## 2. Invariants & Mathematical Properties
/// - Zero Hash Non-Determinism: Backed exclusively by `BTreeMap<(u32, String), ClassifiedDocTierItem>`
///   and `BTreeSet<usize>` (`FM-BP08-006`).
/// - Exact Partition Identity: `production_items.len() == rust_tier_a_count + rust_tier_bc_count` and
///   `production_exec_sloc == production_exec_lines.len()`.
/// - Complete Test Pruning: Zero `#[cfg(test)]` or `#[test]` subtree items or executable lines appear
///   in `production_items` or `production_exec_lines`; pruned items increment `test_excluded_item_count`.
///
/// ## 3. Preconditions
/// - Synthesized exclusively by `classify_cfg_test_excluded_doc_tier` from a sealed `VerifiedLosslessCst`
///   and validated `CstMetricsThresholds`.
///
/// ## 4. Postconditions
/// - Provides immutable `BTreeMap`-ordered tier assignments, pruned test subtree byte spans, and
///   `production_exec_sloc` accounting to downstream `v2_metrics` and Check-36 stages.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(log I) lookup over I production items; O(1) aggregate counter reads.
/// - Space Complexity: O(I + L + T) for I production items, L executable lines, and T pruned spans.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Exposes strongly typed ordered collections and checked `usize` counters.
/// - State: Self-contained value record with deterministic `PartialEq` / `Eq` semantics.
/// - Action: Suppresses partial publication if any `DEAP-E0801`..`DEAP-E0804` fault occurs.
/// - Resource: Bounded by verified compilation unit size and `MAX_AST_DEPTH = 64`.
///
/// ## 7. Realization & Traceability
/// Realises: [LosslessRustCstAnalyzer/DocTierInventory] | [REQ-0608] | [FLS: fls_ikfvbeewame7] | [FLS: fls_nsvzzbldhq53] | SysML v2: part def LosslessRustCstAnalyzer
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DocTierInventory {
    /// Canonical relative or logical path of the classified Rust compilation unit.
    /// Realises: [REQ-0608]
    pub source_path: String,
    /// Deterministic `BTreeMap` of non-test production items keyed by `(byte_start_u32, symbol)`.
    /// Realises: [REQ-0608]
    pub production_items: BTreeMap<(u32, String), ClassifiedDocTierItem>,
    /// Ordered 0-based executable line indices outside `#[cfg(test)]`, `#[test]`, trivia, and `doc` attrs.
    /// Realises: [REQ-0608]
    pub production_exec_lines: BTreeSet<usize>,
    /// Total production executable SLOC (`production_exec_lines.len()`) after pruning test subtrees.
    /// Realises: [REQ-0608]
    pub production_exec_sloc: usize,
    /// Count of `DocTier::RustTierA` items recorded in `production_items`.
    /// Realises: [REQ-0608]
    pub rust_tier_a_count: usize,
    /// Count of `DocTier::RustTierBC` items recorded in `production_items`.
    /// Realises: [REQ-0608]
    pub rust_tier_bc_count: usize,
    /// Audit counter of `DocTier::TestExcluded` items pruned inside `#[cfg(test)]` and `#[test]` subtrees.
    /// Realises: [REQ-0608]
    pub test_excluded_item_count: usize,
    /// Half-open UTF-8 `(byte_start, byte_end)` spans of pruned `#[cfg(test)]` and `#[test]` subtree roots.
    /// Realises: [REQ-0608]
    pub pruned_test_subtree_ranges: Vec<(usize, usize)>,
}

/// Initializes an empty `DocTierInventory` bound to `source_path`.
/// Realises: [REQ-0608]
fn new_empty_doc_tier_inventory(source_path: &str) -> DocTierInventory {
    DocTierInventory {
        source_path: source_path.to_string(),
        production_items: BTreeMap::new(),
        production_exec_lines: BTreeSet::new(),
        production_exec_sloc: 0,
        rust_tier_a_count: 0,
        rust_tier_bc_count: 0,
        test_excluded_item_count: 0,
        pruned_test_subtree_ranges: Vec::new(),
    }
}

/// Returns `true` if `tok` is whitespace or a normal/doc comment trivia token.
/// Realises: [REQ-0608]
fn is_trivia_token(tok: &ra_ap_syntax::SyntaxToken) -> bool {
    matches!(
        tok.kind(),
        ra_ap_syntax::SyntaxKind::WHITESPACE
            | ra_ap_syntax::SyntaxKind::COMMENT
            | ra_ap_syntax::SyntaxKind::OUTER_DOC_COMMENT
            | ra_ap_syntax::SyntaxKind::INNER_DOC_COMMENT
    )
}

/// Collects non-trivia leaf `SyntaxToken` elements inside `node` up to `limit` tokens.
/// Realises: [REQ-0608]
fn collect_bounded_non_trivia_tokens(
    node: &ra_ap_syntax::SyntaxNode,
    limit: usize,
) -> Vec<ra_ap_syntax::SyntaxToken> {
    let mut out = Vec::new();
    for elem in node.descendants_with_tokens() {
        if let ra_ap_syntax::SyntaxElement::Token(tok) = elem {
            if !is_trivia_token(&tok) {
                out.push(tok);
                if out.len() >= limit {
                    break;
                }
            }
        }
    }
    out
}

/// Returns `true` iff `attr` is an unqualified `doc` attribute (`#[doc = ...]`, `#![doc = ...]`, `#[doc(...)]`).
/// Realises: [REQ-0608]
fn is_doc_meta_attr(attr: &ra_ap_syntax::ast::Attr) -> bool {
    let Some(meta) = attr.meta() else {
        return false;
    };
    let tokens = collect_bounded_non_trivia_tokens(meta.syntax(), 3);
    let first_is_doc = tokens.first().is_some_and(|tok| tok.text() == "doc");
    let second_is_delim = tokens
        .get(1)
        .is_some_and(|tok| tok.text() == "=" || tok.text() == "(");
    first_is_doc && second_is_delim
}

/// Returns `true` if `node` or any bounded CST ancestor is a `doc` attribute node.
/// Realises: [REQ-0608]
fn is_inside_doc_attr(node: &ra_ap_syntax::SyntaxNode) -> bool {
    for ancestor in node.ancestors().take(MAX_AST_DEPTH.saturating_add(1)) {
        if let Some(attr) = ra_ap_syntax::ast::Attr::cast(ancestor) {
            if is_doc_meta_attr(&attr) {
                return true;
            }
        }
    }
    false
}

/// Returns `true` iff `attr` is an outer `#[cfg(test)]` attribute (excluding `#[cfg(not(test))]`).
/// Realises: [REQ-0608]
fn is_outer_cfg_test_attr(attr: &ra_ap_syntax::ast::Attr) -> bool {
    if attr.kind() != ra_ap_syntax::ast::AttrKind::Outer {
        return false;
    }
    let Some(meta) = attr.meta() else {
        return false;
    };
    let tokens = collect_bounded_non_trivia_tokens(meta.syntax(), 5);
    if tokens.len() != 4 {
        return false;
    }
    let expected = ["cfg", "(", "test", ")"];
    let mut idx = 0usize;
    while let Some(&exp) = expected.get(idx) {
        if tokens.get(idx).map(|tok| tok.text()) != Some(exp) {
            return false;
        }
        idx = idx.saturating_add(1);
    }
    true
}

/// Returns `true` iff `attr` is an outer `#[test]` or `#[...::test]` callable path attribute.
/// Realises: [REQ-0608]
fn is_outer_test_fn_attr(attr: &ra_ap_syntax::ast::Attr) -> bool {
    if attr.kind() != ra_ap_syntax::ast::AttrKind::Outer {
        return false;
    }
    let Some(meta) = attr.meta() else {
        return false;
    };
    let tokens = collect_bounded_non_trivia_tokens(meta.syntax(), 16);
    if tokens.is_empty() || tokens.len() >= 16 {
        return false;
    }
    let all_path_tokens = tokens.iter().all(|tok| {
        matches!(
            tok.kind(),
            ra_ap_syntax::SyntaxKind::IDENT
                | ra_ap_syntax::SyntaxKind::COLON2
                | ra_ap_syntax::SyntaxKind::CRATE_KW
                | ra_ap_syntax::SyntaxKind::SELF_KW
                | ra_ap_syntax::SyntaxKind::SUPER_KW
        )
    });
    all_path_tokens && tokens.last().is_some_and(|tok| tok.text() == "test")
}

/// Returns `true` if `node` carries an outer `#[cfg(test)]` attribute or is a `#[test]` `ast::Fn`.
/// Realises: [REQ-0608]
fn node_enters_test_scope(node: &ra_ap_syntax::SyntaxNode) -> bool {
    if node
        .children()
        .filter_map(ra_ap_syntax::ast::Attr::cast)
        .any(|attr| is_outer_cfg_test_attr(&attr))
    {
        return true;
    }
    if let Some(fn_item) = ra_ap_syntax::ast::Fn::cast(node.clone()) {
        return ra_ap_syntax::ast::HasAttrs::attrs(&fn_item)
            .any(|attr| is_outer_test_fn_attr(&attr));
    }
    false
}

/// Returns `true` iff `vis` is an unrestricted `pub` modifier (excluding `pub(crate)` / `pub(...)`).
/// Realises: [REQ-0608]
fn is_unrestricted_pub_visibility(vis: Option<ra_ap_syntax::ast::Visibility>) -> bool {
    let Some(vis_node) = vis else {
        return false;
    };
    if vis_node.pub_token().is_none() {
        return false;
    }
    !vis_node
        .syntax()
        .descendants_with_tokens()
        .any(|el| el.kind() == ra_ap_syntax::SyntaxKind::L_PAREN)
}

/// Returns `true` when `type_alias` is an associated `type` binding inside `impl Trait for Type`.
/// Realises: [REQ-0608]
fn is_trait_impl_assoc_type(type_alias: &ra_ap_syntax::ast::TypeAlias) -> bool {
    let Some(parent) = ra_ap_syntax::AstNode::syntax(type_alias).parent() else {
        return false;
    };
    let Some(grandparent) = parent.parent() else {
        return false;
    };
    ra_ap_syntax::ast::Impl::cast(grandparent).is_some_and(|imp| imp.trait_().is_some())
}

/// Extracts `(symbol, is_unrestricted_pub, is_callable)` from a named visibility-bearing AST node.
/// Realises: [REQ-0608]
fn item_name_and_pub<T>(item: &T, is_callable: bool) -> Option<(String, bool, bool)>
where
    T: ra_ap_syntax::ast::HasName + ra_ap_syntax::ast::HasVisibility,
{
    let symbol = item.name()?.text().to_string();
    let is_pub = is_unrestricted_pub_visibility(item.visibility());
    Some((symbol, is_pub, is_callable))
}

/// Extracts `(symbol, is_unrestricted_pub, is_callable)` for named `DocTier`-eligible Rust CST items.
/// Realises: [REQ-0608]
fn extract_named_doc_tier_target(
    node: &ra_ap_syntax::SyntaxNode,
    inside_test_scope: bool,
) -> Option<(String, bool, bool)> {
    use ra_ap_syntax::ast::Item;
    match Item::cast(node.clone())? {
        Item::Fn(it) => item_name_and_pub(&it, true),
        Item::Struct(it) => item_name_and_pub(&it, false),
        Item::Enum(it) => item_name_and_pub(&it, false),
        Item::Union(it) => item_name_and_pub(&it, false),
        Item::Trait(it) => item_name_and_pub(&it, false),
        Item::Const(it) => item_name_and_pub(&it, false),
        Item::Static(it) => item_name_and_pub(&it, false),
        Item::TypeAlias(it) if inside_test_scope || !is_trait_impl_assoc_type(&it) => {
            item_name_and_pub(&it, false)
        }
        Item::Module(it) if inside_test_scope || it.item_list().is_some() => {
            item_name_and_pub(&it, false)
        }
        _ => None,
    }
}

/// Returns the outer documentation line count contributed by a single `COMMENT` token slice.
/// Realises: [REQ-0608]
fn outer_doc_comment_line_delta(slice: &str) -> usize {
    if slice.starts_with("///") && !slice.starts_with("////") {
        return 1;
    }
    if slice.starts_with("/**") && !slice.starts_with("/***") && slice != "/**/" {
        return slice.lines().count().max(1);
    }
    0
}

/// Slices `source_text` via `.get(start..end)` and adds `outer_doc_comment_line_delta` via `checked_add`.
/// Realises: [REQ-0608]
fn checked_outer_doc_slice_lines(
    source_text: &str,
    start: usize,
    end: usize,
    current: usize,
) -> Result<usize, CstMetricsError> {
    let slice =
        source_text
            .get(start..end)
            .ok_or(CstMetricsError::LosslessRoundtripOrSpanMismatch {
                byte_start: start,
                byte_end: end,
            })?;
    current
        .checked_add(outer_doc_comment_line_delta(slice))
        .ok_or_else(|| depth_exhaustion_err(usize::MAX, MAX_AST_DEPTH, start))
}

/// Counts leading outer `///` / `/** */` doc lines and `#[doc = ...]` attributes after UTF-8 boundary checks.
/// Realises: [REQ-0608]
fn extract_verified_outer_doc_line_count(
    node: &ra_ap_syntax::SyntaxNode,
    source_text: &str,
) -> Result<usize, CstMetricsError> {
    let mut doc_lines = 0usize;
    for child in node.children_with_tokens() {
        let range = child.text_range();
        let start = u32::from(range.start()) as usize;
        let end = u32::from(range.end()) as usize;
        verify_utf8_char_boundary_span(source_text, start, end)?;
        match child {
            ra_ap_syntax::SyntaxElement::Token(tok) if is_trivia_token(&tok) => {
                if tok.kind() == ra_ap_syntax::SyntaxKind::OUTER_DOC_COMMENT {
                    doc_lines = checked_outer_doc_slice_lines(source_text, start, end, doc_lines)?;
                }
            }
            ra_ap_syntax::SyntaxElement::Node(child_node)
                if child_node.kind() == ra_ap_syntax::SyntaxKind::DOC_COMMENT =>
            {
                doc_lines = checked_outer_doc_slice_lines(source_text, start, end, doc_lines)?;
            }
            ra_ap_syntax::SyntaxElement::Node(child_node) => {
                let Some(attr) = ra_ap_syntax::ast::Attr::cast(child_node) else {
                    break;
                };
                if attr.kind() == ra_ap_syntax::ast::AttrKind::Outer && is_doc_meta_attr(&attr) {
                    doc_lines = doc_lines
                        .checked_add(1)
                        .ok_or_else(|| depth_exhaustion_err(usize::MAX, MAX_AST_DEPTH, start))?;
                }
            }
            _ => break,
        }
    }
    Ok(doc_lines)
}

/// Resolves a UTF-8 `byte_offset` to its 0-based line index via `binary_search` and `.get(...)`.
/// Realises: [REQ-0608]
fn resolve_byte_to_line_index(
    line_starts: &[usize],
    byte_offset: usize,
) -> Result<usize, CstMetricsError> {
    let line_idx =
        match line_starts.binary_search(&byte_offset) {
            Ok(exact_idx) => exact_idx,
            Err(insert_pos) => insert_pos.checked_sub(1).ok_or(
                CstMetricsError::LosslessRoundtripOrSpanMismatch {
                    byte_start: byte_offset,
                    byte_end: byte_offset,
                },
            )?,
        };
    if line_starts.get(line_idx).is_none() {
        return Err(CstMetricsError::LosslessRoundtripOrSpanMismatch {
            byte_start: byte_offset,
            byte_end: byte_offset,
        });
    }
    Ok(line_idx)
}

/// Inserts 0-based line indices covered by direct non-trivia `SyntaxToken` children of `node` into `exec_lines`.
/// Realises: [REQ-0608]
fn record_node_direct_exec_lines(
    node: &ra_ap_syntax::SyntaxNode,
    source_text: &str,
    line_starts: &[usize],
    exec_lines: &mut BTreeSet<usize>,
) -> Result<(), CstMetricsError> {
    if is_inside_doc_attr(node) {
        return Ok(());
    }
    for child in node.children_with_tokens() {
        let ra_ap_syntax::SyntaxElement::Token(tok) = child else {
            continue;
        };
        if is_trivia_token(&tok) {
            continue;
        }
        let range = tok.text_range();
        let (start, end) = (
            u32::from(range.start()) as usize,
            u32::from(range.end()) as usize,
        );
        verify_utf8_char_boundary_span(source_text, start, end)?;
        if let Some(last_byte) = end.checked_sub(1) {
            if start <= last_byte {
                let first_line = resolve_byte_to_line_index(line_starts, start)?;
                let last_line = resolve_byte_to_line_index(line_starts, last_byte)?;
                let mut cursor = first_line;
                while cursor <= last_line {
                    exec_lines.insert(cursor);
                    cursor = cursor
                        .checked_add(1)
                        .ok_or_else(|| depth_exhaustion_err(usize::MAX, MAX_AST_DEPTH, start))?;
                }
            }
        }
    }
    Ok(())
}

/// Maps `(is_unrestricted_pub, promote_self_hosted)` for a non-test item onto `RustTierA` or `RustTierBC`.
/// Realises: [REQ-0608]
const fn select_production_doc_tier(
    is_unrestricted_pub: bool,
    promote_self_hosted: bool,
) -> DocTier {
    if is_unrestricted_pub || promote_self_hosted {
        DocTier::RustTierA
    } else {
        DocTier::RustTierBC
    }
}

/// Inserts a verified non-test `ClassifiedDocTierItem` into `inventory` and increments tier counters via `checked_add`.
/// Realises: [REQ-0608]
fn insert_production_tier_item(
    inventory: &mut DocTierInventory,
    start_u32: u32,
    item: ClassifiedDocTierItem,
) -> Result<(), CstMetricsError> {
    let (byte_start, byte_end) = (item.byte_start, item.byte_end);
    match item.tier {
        DocTier::RustTierA => {
            inventory.rust_tier_a_count = inventory
                .rust_tier_a_count
                .checked_add(1)
                .ok_or_else(|| depth_exhaustion_err(usize::MAX, MAX_AST_DEPTH, byte_start))?;
        }
        DocTier::RustTierBC => {
            inventory.rust_tier_bc_count = inventory
                .rust_tier_bc_count
                .checked_add(1)
                .ok_or_else(|| depth_exhaustion_err(usize::MAX, MAX_AST_DEPTH, byte_start))?;
        }
        DocTier::TestExcluded => {
            return Err(CstMetricsError::UnverifiedParsePrecondition {
                state: "AnalyzingMetricsAndTiers",
            });
        }
    }
    let key = (start_u32, item.symbol.clone());
    if inventory.production_items.insert(key, item).is_some() {
        return Err(CstMetricsError::LosslessRoundtripOrSpanMismatch {
            byte_start,
            byte_end,
        });
    }
    Ok(())
}

/// Classifies `node` (when `DocTier`-eligible) into `TestExcluded`, `RustTierA`, or `RustTierBC` inside `inventory`.
/// Realises: [REQ-0608]
fn record_classified_node_in_inventory(
    node: &ra_ap_syntax::SyntaxNode,
    source_text: &str,
    effective_test_scope: bool,
    promote_self_hosted: bool,
    inventory: &mut DocTierInventory,
) -> Result<(), CstMetricsError> {
    let Some((symbol, is_pub, is_callable)) =
        extract_named_doc_tier_target(node, effective_test_scope)
    else {
        return Ok(());
    };
    let range = node.text_range();
    let byte_start = u32::from(range.start()) as usize;
    let byte_end = u32::from(range.end()) as usize;
    let doc_line_count = extract_verified_outer_doc_line_count(node, source_text)?;
    if effective_test_scope {
        inventory.test_excluded_item_count = inventory
            .test_excluded_item_count
            .checked_add(1)
            .ok_or_else(|| depth_exhaustion_err(usize::MAX, MAX_AST_DEPTH, byte_start))?;
        return Ok(());
    }
    let tier = select_production_doc_tier(is_pub, promote_self_hosted);
    let record = ClassifiedDocTierItem {
        symbol,
        tier,
        byte_start,
        byte_end,
        doc_line_count,
        is_callable,
    };
    insert_production_tier_item(inventory, u32::from(range.start()), record)
}

/// Pushes direct child `SyntaxNode` elements of `node` onto `stack` in reverse lexical order with checked depth.
/// Realises: [REQ-0608]
fn push_tier_worklist_children(
    node: &ra_ap_syntax::SyntaxNode,
    depth: usize,
    max_depth: usize,
    inside_test_scope: bool,
    stack: &mut Vec<(ra_ap_syntax::SyntaxNode, usize, bool)>,
) -> Result<(), CstMetricsError> {
    let children: Vec<ra_ap_syntax::SyntaxNode> = node.children().collect();
    for child in children.into_iter().rev() {
        let offset = u32::from(child.text_range().start()) as usize;
        let next_depth = checked_next_cst_depth(depth, max_depth, offset)?;
        stack.push((child, next_depth, inside_test_scope));
    }
    Ok(())
}

/// Validates `node` UTF-8 boundaries, updates `inventory` tiers/SLOC, and returns `effective_test_scope`.
/// Realises: [REQ-0608]
fn visit_tier_worklist_node(
    node: &ra_ap_syntax::SyntaxNode,
    cst: &VerifiedLosslessCst,
    inside_test_scope: bool,
    promote_self_hosted: bool,
    inventory: &mut DocTierInventory,
) -> Result<bool, CstMetricsError> {
    let range = node.text_range();
    let start = u32::from(range.start()) as usize;
    let end = u32::from(range.end()) as usize;
    let source_text = cst.source_text();
    verify_utf8_char_boundary_span(source_text, start, end)?;
    let enters_test = !inside_test_scope && node_enters_test_scope(node);
    if enters_test {
        inventory.pruned_test_subtree_ranges.push((start, end));
    }
    let effective_test_scope = inside_test_scope || enters_test;
    record_classified_node_in_inventory(
        node,
        source_text,
        effective_test_scope,
        promote_self_hosted,
        inventory,
    )?;
    if !effective_test_scope {
        record_node_direct_exec_lines(
            node,
            source_text,
            cst.line_starts(),
            &mut inventory.production_exec_lines,
        )?;
    }
    Ok(effective_test_scope)
}

/// Prunes `#[cfg(test)]` and `#[test]` CST subtrees and classifies production items into `DocTier` (`MT-US56-03`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C structural
/// complexity and documentation tier governance (`classify_cfg_test_excluded_doc_tier`, MT-US56-03,
/// US-56 / FEAT-42, REQ-0608, FLS `fls_ikfvbeewame7`, FLS `fls_nsvzzbldhq53`). Executes the
/// `AnalyzingMetricsAndTiers` subtree-pruning pass so test modules and helpers never contaminate
/// `DocTier::RustTierA` / `DocTier::RustTierBC` inventories or `production_exec_sloc` budgets.
///
/// ## 2. Invariants & Mathematical Properties
/// - Subtree-Scoped Test Exclusion: Any node bearing outer `#[cfg(test)]` or `#[test]` (`ast::Fn`)
///   transitions its descendant worklist frames to `inside_test_scope == true`, pruning 100% of
///   descendant tokens from `production_exec_lines` and recording items exclusively in
///   `test_excluded_item_count` (`SC-BP08-06`, `FM-BP08-006`).
/// - Tier Assignment: Unrestricted `pub` items and generated self-hosted items (`is_generated_self_hosted`
///   or `.g.rs` source path) map to `DocTier::RustTierA`; restricted `pub(...)` and private
///   non-test items map to `DocTier::RustTierBC`.
/// - Complexity Budget Compliance: `3` formal parameters, `<= 46` executable SLOC, `0` mutable
///   `bool` flags, compound boolean predicate arity `<= 2`, and iterative `MAX_AST_DEPTH = 64` guard.
///
/// ## 3. Preconditions
/// - `cst` references a sealed `VerifiedLosslessCst` (`is_lossless_red_green_tree_ready() == true`).
/// - `thresholds` satisfies `validate_metrics_thresholds` (`C_CST_01`..`C_CST_03`, `MAX_AST_DEPTH = 64`).
/// - `is_generated_self_hosted` specifies whether all non-test items belong to a generated self-hosted unit.
///
/// ## 4. Postconditions
/// - Returns `Ok(DocTierInventory)` ordered deterministically in `BTreeMap` / `BTreeSet` structures.
/// - Returns `Err(CstMetricsError)` (`DEAP-E0801`..`DEAP-E0804`) fail-closed on unsealed CST state,
///   round-trip/UTF-8 boundary mismatch, threshold drift, or `MAX_AST_DEPTH = 64` exhaustion.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N log L + I log I) over N CST elements, L source lines, and I production items.
/// - Space Complexity: O(N + I + L) for the bounded iterative worklist stack and `BTreeMap`/`BTreeSet` sets.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Total function taking `3` parameters and returning typed `Result<DocTierInventory, CstMetricsError>`.
/// - State: Pure read-only CST analysis performing zero global or filesystem mutations.
/// - Action: Aborts fail-closed without partial inventory emission on `DEAP-E0801`..`DEAP-E0804` faults.
/// - Resource: Explicit heap worklist `(SyntaxNode, usize, bool)` bounded by `MAX_AST_DEPTH = 64`.
///
/// ## 7. Realization & Traceability
/// Realises: [LosslessRustCstAnalyzer/classify_cfg_test_excluded_doc_tier] | [REQ-0608] | [FLS: fls_ikfvbeewame7] | [FLS: fls_nsvzzbldhq53] | SysML v2: part def LosslessRustCstAnalyzer
pub fn classify_cfg_test_excluded_doc_tier(
    cst: &VerifiedLosslessCst,
    thresholds: &CstMetricsThresholds,
    is_generated_self_hosted: bool,
) -> Result<DocTierInventory, CstMetricsError> {
    if !cst.is_lossless_red_green_tree_ready() {
        return Err(CstMetricsError::UnverifiedParsePrecondition {
            state: "AnalyzingMetricsAndTiers",
        });
    }
    validate_metrics_thresholds(thresholds)?;
    verify_green_tree_roundtrip_and_depth(
        cst.green_node(),
        cst.source_text(),
        thresholds.max_ast_depth,
    )?;
    let promote_self_hosted = is_generated_self_hosted || cst.source_path().ends_with(".g.rs");
    let mut inventory = new_empty_doc_tier_inventory(cst.source_path());
    let mut stack = vec![(cst.syntax_node().clone(), 0usize, false)];
    while let Some((node, depth, inside_test_scope)) = stack.pop() {
        let start = u32::from(node.text_range().start()) as usize;
        if depth > thresholds.max_ast_depth {
            return Err(depth_exhaustion_err(depth, thresholds.max_ast_depth, start));
        }
        let effective_test_scope = visit_tier_worklist_node(
            &node,
            cst,
            inside_test_scope,
            promote_self_hosted,
            &mut inventory,
        )?;
        push_tier_worklist_children(
            &node,
            depth,
            thresholds.max_ast_depth,
            effective_test_scope,
            &mut stack,
        )?;
    }
    inventory.production_exec_sloc = inventory.production_exec_lines.len();
    Ok(inventory)
}

/// Per-callable `v2_metrics` structural complexity record extracted from a lossless Rust CST (`MT-US56-03`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C structural
/// complexity verification mandates (`FnAstMetrics`, MT-US56-03, US-56 / FEAT-42, REQ-0608,
/// FLS `fls_ikfvbeewame7`, FLS `fls_nsvzzbldhq53`). Captures deterministic per-function executable
/// SLOC (`C_CST_01`), receiver-excluded parameter arity (`C_CST_02`), local mutable `bool` state
/// flags (`C_CST_03`), compound boolean predicate clause arity, `#[rt_loop]` outer-attribute
/// annotations, and `Result<T, String>` return-type violations.
///
/// ## 2. Invariants & Mathematical Properties
/// - Ordered Line Set Parity: `exec_sloc == exec_lines.len()`, backed exclusively by `BTreeSet<usize>`
///   with zero `HashSet` non-determinism (`FM-BP08-006`).
/// - Receiver Exclusion: `self`, `&self`, and `&mut self` (`ast::SelfParam`) are excluded from
///   `param_count` while `has_self_receiver` records receiver presence.
/// - Verified Span Ordering: `byte_start <= byte_end` correspond to the owning `ast::Fn` `TextRange`.
///
/// ## 3. Preconditions
/// - Synthesized by `extract_callable_v2_metrics` over a lossless `ra_ap_syntax::ast::Fn` node and
///   monotone `line_starts` UTF-8 byte-offset table within `MAX_AST_DEPTH = 64`.
///
/// ## 4. Postconditions
/// - Stored in `DeapV2AstMetricsReport::callable_metrics` keyed by `(byte_start as u32, symbol)`
///   for deterministic `C_CST_01`..`C_CST_03` evaluation and single-line `rustc` NDJSON reporting.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(1) scalar field access; O(L) cloning over L executable lines in `exec_lines`.
/// - Space Complexity: O(|symbol| + L) proportional to identifier length and callable SLOC.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Strongly typed immutable value record implementing `Ord`, `Eq`, and `Serialize`.
/// - State: Pure value struct free of interior mutability or global state.
/// - Action: Zero panicking operations.
/// - Resource: Bounded by single-function CST footprint within `MAX_AST_DEPTH = 64`.
///
/// ## 7. Realization & Traceability
/// Realises: [LosslessRustCstAnalyzer/FnAstMetrics] | [REQ-0608] | [FLS: fls_ikfvbeewame7] | [FLS: fls_nsvzzbldhq53] | SysML v2: part def LosslessRustCstAnalyzer
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct FnAstMetrics {
    /// Unqualified callable identifier extracted from the lossless `ra_ap_syntax::ast::Fn` node.
    /// Realises: [REQ-0608]
    pub symbol: String,
    /// Inclusive UTF-8 start byte offset of the callable `SyntaxNode` (`TextRange::start()`).
    /// Realises: [REQ-0608]
    pub byte_start: usize,
    /// Exclusive UTF-8 end byte offset of the callable `SyntaxNode` (`TextRange::end()`).
    /// Realises: [REQ-0608]
    pub byte_end: usize,
    /// Ordered 0-based non-trivia executable line indices covered by this callable (`BTreeSet`).
    /// Realises: [REQ-0608]
    pub exec_lines: BTreeSet<usize>,
    /// Non-trivia executable SLOC count (`exec_lines.len()`) governed by `C_CST_01` (`<= 50`).
    /// Realises: [REQ-0608]
    pub exec_sloc: usize,
    /// Receiver-excluded formal parameter count (excluding `self`, `&self`, `&mut self`, `C_CST_02`).
    /// Realises: [REQ-0608]
    pub param_count: usize,
    /// True when an explicit `self`, `&self`, or `&mut self` receiver parameter was excluded.
    /// Realises: [REQ-0608]
    pub has_self_receiver: bool,
    /// Count of local mutable `bool` bindings (`let mut flag: bool` / `true` / `false`, `C_CST_03`).
    /// Realises: [REQ-0608]
    pub mut_bool_flag_count: usize,
    /// Maximum boolean clause count across logical `&&` and `||` `ast::BinExpr` predicate trees.
    /// Realises: [REQ-0608]
    pub max_predicate_clauses: usize,
    /// True when the callable carries an outer `#[rt_loop]` real-time loop attribute annotation.
    /// Realises: [REQ-0608]
    pub is_rt_loop: bool,
    /// True when the callable return type resolves to `Result<T, String>` instead of a typed error.
    /// Realises: [REQ-0608]
    pub returns_result_string_err: bool,
}

/// Immutable `BTreeMap`-ordered `v2_metrics` and `cfg(test)`-pruned `DocTier` report (`MT-US56-03`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C deterministic
/// syntax analysis mandates (`DeapV2AstMetricsReport`, MT-US56-03, US-56 / FEAT-42, REQ-0608,
/// FLS `fls_ikfvbeewame7`, FLS `fls_nsvzzbldhq53`). Realises the `reply_DeapV2AstMetrics` payload
/// emitted on `LosslessRustCstAnalyzer::metrics_report_out` after `#[cfg(test)]` and `#[test]`
/// subtrees are pruned in `AnalyzingMetricsAndTiers`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Deterministic Ordering: `callable_metrics` is backed exclusively by
///   `BTreeMap<(u32, String), FnAstMetrics>` and `production_exec_lines` by `BTreeSet<usize>`
///   (zero `HashMap` or `HashSet` non-determinism, mitigating `FM-BP08-006`).
/// - Complete Test Subtree Exclusion: Zero `DocTier::TestExcluded` (`#[cfg(test)]` or `#[test]`)
///   callables or lines appear in `callable_metrics`, `production_exec_lines`, or
///   `production_exec_sloc`.
/// - Verified CST Envelope: `observed_max_depth <= MAX_AST_DEPTH` (`64`) and
///   `lossless_roundtrip_verified == true`.
///
/// ## 3. Preconditions
/// - Synthesized exclusively by `analyze_cst_v2_metrics` from a sealed `VerifiedLosslessCst` token
///   and validated `CstMetricsThresholds`.
///
/// ## 4. Postconditions
/// - Supplies deterministic per-callable `FnAstMetrics`, `DocTierInventory`, and production SLOC
///   accounting to `UnifiedRustcJsonDiagnosticEmitter` (`MT-US56-04`).
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(log F) callable lookup over F non-test functions; O(1) aggregate reads.
/// - Space Complexity: O(F + I + L) across F callables, I classified items, and L executable lines.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Strongly typed report value implementing `Clone`, `PartialEq`, `Eq`, and `Serialize`.
/// - State: Immutable value record; partial emission is suppressed on `DEAP-E0801`..`DEAP-E0804` faults.
/// - Action: Zero panicking accessors or unchecked slice operations.
/// - Resource: Bounded by verified compilation unit size and `MAX_AST_DEPTH = 64`.
///
/// ## 7. Realization & Traceability
/// Realises: [LosslessRustCstAnalyzer/DeapV2AstMetricsReport] | [REQ-0608] | [FLS: fls_ikfvbeewame7] | [FLS: fls_nsvzzbldhq53] | SysML v2: part def LosslessRustCstAnalyzer
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DeapV2AstMetricsReport {
    /// Canonical relative or logical path of the analyzed Rust compilation unit.
    /// Realises: [REQ-0608]
    pub source_path: String,
    /// Deterministic `BTreeMap` of non-test callable `v2_metrics` keyed by `(byte_start_u32, symbol)`.
    /// Realises: [REQ-0608]
    pub callable_metrics: BTreeMap<(u32, String), FnAstMetrics>,
    /// Deterministic `cfg(test)`-pruned `DocTier` item inventory (`RustTierA` vs `RustTierBC`).
    /// Realises: [REQ-0608]
    pub doc_tier_inventory: DocTierInventory,
    /// Ordered 0-based production executable line indices after pruning `#[cfg(test)]` and `#[test]`.
    /// Realises: [REQ-0608]
    pub production_exec_lines: BTreeSet<usize>,
    /// Total production executable SLOC (`production_exec_lines.len()`) outside test subtrees.
    /// Realises: [REQ-0608]
    pub production_exec_sloc: usize,
    /// Count of non-test `DocTier::RustTierA` declarations in `doc_tier_inventory`.
    /// Realises: [REQ-0608]
    pub rust_tier_a_count: usize,
    /// Count of non-test `DocTier::RustTierBC` declarations in `doc_tier_inventory`.
    /// Realises: [REQ-0608]
    pub rust_tier_bc_count: usize,
    /// Audit count of pruned `DocTier::TestExcluded` items inside `#[cfg(test)]` and `#[test]` subtrees.
    /// Realises: [REQ-0608]
    pub test_excluded_item_count: usize,
    /// Maximum `SyntaxNode` CST nesting depth observed across the compilation unit (`<= 64`).
    /// Realises: [REQ-0608]
    pub observed_max_depth: usize,
    /// True confirming bit-for-bit root `rowan::GreenNode` round-trip and UTF-8 boundary parity.
    /// Realises: [REQ-0608]
    pub lossless_roundtrip_verified: bool,
}

/// Mutable per-callable CST accumulator for executable lines, `mut bool` flags, and predicate clauses.
/// Realises: [REQ-0608]
struct CallableBodyAccumulator {
    /// Ordered 0-based non-trivia executable line indices covered by the callable.
    /// Realises: [REQ-0608]
    exec_lines: BTreeSet<usize>,
    /// Accumulated count of local mutable `bool` bindings (`C_CST_03`).
    /// Realises: [REQ-0608]
    mut_bool_flag_count: usize,
    /// Maximum logical `&&` / `||` clause count observed inside the callable.
    /// Realises: [REQ-0608]
    max_predicate_clauses: usize,
}

/// Extracts `(symbol, byte_start, byte_end)` from `fn_node` or fails closed with `DEAP-E0802`/`DEAP-E0803`.
/// Realises: [REQ-0608]
fn extract_fn_symbol_and_span(
    fn_node: &ra_ap_syntax::ast::Fn,
) -> Result<(String, usize, usize), CstMetricsError> {
    let range = fn_node.syntax().text_range();
    let byte_start = u32::from(range.start()) as usize;
    let byte_end = u32::from(range.end()) as usize;
    if byte_start > byte_end {
        return Err(CstMetricsError::LosslessRoundtripOrSpanMismatch {
            byte_start,
            byte_end,
        });
    }
    let symbol = ra_ap_syntax::ast::HasName::name(fn_node)
        .map(|name| name.text().to_string())
        .ok_or_else(|| CstMetricsError::MalformedUtf8OrRustSyntax {
            byte_start,
            byte_end,
            detail: "callable ast::Fn is missing identifier".to_string(),
        })?;
    Ok((symbol, byte_start, byte_end))
}

/// Computes the 0-based CST ancestor depth of `fn_node` and enforces `MAX_AST_DEPTH = 64` (`DEAP-E0804`).
/// Realises: [REQ-0608]
fn compute_fn_root_cst_depth(fn_node: &ra_ap_syntax::ast::Fn) -> Result<usize, CstMetricsError> {
    let start = u32::from(fn_node.syntax().text_range().start()) as usize;
    let mut depth = 0usize;
    let mut cursor = fn_node.syntax().parent();
    while let Some(parent_node) = cursor {
        depth = checked_next_cst_depth(depth, MAX_AST_DEPTH, start)?;
        cursor = parent_node.parent();
    }
    Ok(depth)
}

/// Extracts `(param_count, has_self_receiver)` excluding `self`, `&self`, and `&mut self` receivers.
/// Realises: [REQ-0608]
fn extract_receiver_excluded_arity(
    fn_node: &ra_ap_syntax::ast::Fn,
) -> Result<(usize, bool), CstMetricsError> {
    let Some(param_list) = fn_node.param_list() else {
        return Ok((0, false));
    };
    let has_self_receiver = param_list.self_param().is_some();
    let start = u32::from(param_list.syntax().text_range().start()) as usize;
    let mut param_count = 0usize;
    for _param in param_list.params() {
        param_count = param_count
            .checked_add(1)
            .ok_or_else(|| depth_exhaustion_err(usize::MAX, MAX_AST_DEPTH, start))?;
    }
    Ok((param_count, has_self_receiver))
}

/// Returns `true` iff `fn_node` carries an outer `#[rt_loop]` or `#[...::rt_loop]` attribute.
/// Realises: [REQ-0608]
fn has_outer_rt_loop_attr(fn_node: &ra_ap_syntax::ast::Fn) -> bool {
    ra_ap_syntax::ast::HasAttrs::attrs(fn_node).any(|attr| {
        if attr.kind() != ra_ap_syntax::ast::AttrKind::Outer {
            return false;
        }
        attr.meta()
            .and_then(|meta| meta.path())
            .and_then(|path| path.segment())
            .and_then(|seg| seg.name_ref())
            .is_some_and(|name_ref| name_ref.text() == "rt_loop")
    })
}

/// Strips enclosing parenthesized `ast::ParenType` wrappers iteratively up to `MAX_AST_DEPTH` levels.
/// Realises: [REQ-0608]
fn unwrap_paren_ast_type(mut ty: ra_ap_syntax::ast::Type) -> ra_ap_syntax::ast::Type {
    let mut steps = 0usize;
    while let ra_ap_syntax::ast::Type::ParenType(paren) = ty {
        let Some(inner) = paren.ty() else {
            return ra_ap_syntax::ast::Type::ParenType(paren);
        };
        ty = inner;
        steps = steps.saturating_add(1);
        if steps > MAX_AST_DEPTH {
            break;
        }
    }
    ty
}

/// Returns `true` iff `ty` resolves to owned `String` (`String`, `std::string::String`, `alloc::string::String`).
/// Realises: [REQ-0608]
fn is_rust_string_ast_type(ty: ra_ap_syntax::ast::Type) -> bool {
    use ra_ap_syntax::ast::HasGenericArgs;
    let ra_ap_syntax::ast::Type::PathType(path_ty) = unwrap_paren_ast_type(ty) else {
        return false;
    };
    let Some(path) = path_ty.path() else {
        return false;
    };
    let Some(final_seg) = path.segment() else {
        return false;
    };
    if final_seg.generic_arg_list().is_some() {
        return false;
    }
    let is_string_name = final_seg
        .name_ref()
        .is_some_and(|name_ref| name_ref.text() == "String");
    let qualifiers_valid = path.segments().all(|seg| {
        seg.generic_arg_list().is_none()
            && seg
                .name_ref()
                .is_some_and(|n| matches!(n.text(), "String" | "string" | "std" | "alloc"))
    });
    is_string_name && qualifiers_valid
}

/// Returns `true` iff `fn_node` declares a `Result<T, String>` return type in `ast::RetType`.
/// Realises: [REQ-0608]
fn fn_returns_result_string_err(fn_node: &ra_ap_syntax::ast::Fn) -> bool {
    use ra_ap_syntax::ast::HasGenericArgs;
    let Some(ret_ty) = fn_node.ret_type().and_then(|rt| rt.ty()) else {
        return false;
    };
    let ra_ap_syntax::ast::Type::PathType(path_ty) = unwrap_paren_ast_type(ret_ty) else {
        return false;
    };
    let Some(final_seg) = path_ty.path().and_then(|p| p.segment()) else {
        return false;
    };
    if !final_seg.name_ref().is_some_and(|n| n.text() == "Result") {
        return false;
    }
    let Some(arg_list) = final_seg.generic_arg_list() else {
        return false;
    };
    let type_args: Vec<ra_ap_syntax::ast::Type> = arg_list
        .generic_args()
        .filter_map(|arg| match arg {
            ra_ap_syntax::ast::GenericArg::TypeArg(ta) => ta.ty(),
            _ => None,
        })
        .collect();
    type_args.len() == 2
        && type_args
            .get(1)
            .cloned()
            .is_some_and(is_rust_string_ast_type)
}

/// Returns `true` iff `ty` resolves to primitive `bool` (`bool`, `core::primitive::bool`, `std::primitive::bool`).
/// Realises: [REQ-0608]
fn is_bool_ast_type(ty: ra_ap_syntax::ast::Type) -> bool {
    use ra_ap_syntax::ast::HasGenericArgs;
    let ra_ap_syntax::ast::Type::PathType(path_ty) = unwrap_paren_ast_type(ty) else {
        return false;
    };
    let Some(path) = path_ty.path() else {
        return false;
    };
    let Some(final_seg) = path.segment() else {
        return false;
    };
    let is_bool_name = final_seg.generic_arg_list().is_none()
        && final_seg.name_ref().is_some_and(|n| n.text() == "bool");
    let qualifiers_valid = path.segments().all(|seg| {
        seg.generic_arg_list().is_none()
            && seg
                .name_ref()
                .is_some_and(|n| matches!(n.text(), "bool" | "primitive" | "core" | "std"))
    });
    is_bool_name && qualifiers_valid
}

/// Returns `true` iff `expr` (unwrapping parentheses and `!`) is a boolean literal `true` or `false`.
/// Realises: [REQ-0608]
fn is_bool_literal_expr(mut expr: ra_ap_syntax::ast::Expr) -> bool {
    let mut steps = 0usize;
    loop {
        match expr {
            ra_ap_syntax::ast::Expr::ParenExpr(paren) => {
                let Some(inner) = paren.expr() else {
                    return false;
                };
                expr = inner;
            }
            ra_ap_syntax::ast::Expr::PrefixExpr(pref)
                if pref.op_kind() == Some(ra_ap_syntax::ast::UnaryOp::Not) =>
            {
                let Some(inner) = pref.expr() else {
                    return false;
                };
                expr = inner;
            }
            ra_ap_syntax::ast::Expr::Literal(lit) => {
                return matches!(
                    lit.token().kind(),
                    ra_ap_syntax::SyntaxKind::TRUE_KW | ra_ap_syntax::SyntaxKind::FALSE_KW
                );
            }
            _ => return false,
        }
        steps = steps.saturating_add(1);
        if steps > MAX_AST_DEPTH {
            return false;
        }
    }
}

/// Counts positional mutable boolean bindings inside a destructuring `let (...) = ...` statement.
/// Realises: [REQ-0608]
fn count_tuple_let_mut_bool_flags(
    tuple_pat: &ra_ap_syntax::ast::TuplePat,
    ty_opt: Option<ra_ap_syntax::ast::Type>,
    init_opt: Option<ra_ap_syntax::ast::Expr>,
) -> usize {
    let ty_fields: Vec<ra_ap_syntax::ast::Type> = match ty_opt.map(unwrap_paren_ast_type) {
        Some(ra_ap_syntax::ast::Type::TupleType(tt)) => tt.fields().collect(),
        _ => Vec::new(),
    };
    let init_fields: Vec<ra_ap_syntax::ast::Expr> = match init_opt {
        Some(ra_ap_syntax::ast::Expr::TupleExpr(te)) => te.fields().collect(),
        _ => Vec::new(),
    };
    let mut count = 0usize;
    for (idx, field_pat) in tuple_pat.fields().enumerate() {
        if let ra_ap_syntax::ast::Pat::IdentPat(id) = field_pat {
            if id.mut_token().is_some() {
                let f_ty = ty_fields.get(idx).cloned().is_some_and(is_bool_ast_type);
                let f_init = init_fields
                    .get(idx)
                    .cloned()
                    .is_some_and(is_bool_literal_expr);
                if f_ty || f_init {
                    count = count.saturating_add(1);
                }
            }
        }
    }
    count
}

/// Counts mutable boolean bindings declared by a single `ast::LetStmt` (`C_CST_03`).
/// Realises: [REQ-0608]
fn count_let_stmt_mut_bool_flags(let_stmt: &ra_ap_syntax::ast::LetStmt) -> usize {
    let Some(pat) = let_stmt.pat() else {
        return 0;
    };
    match pat {
        ra_ap_syntax::ast::Pat::IdentPat(ident_pat) if ident_pat.mut_token().is_some() => {
            let ty_bool = let_stmt.ty().is_some_and(is_bool_ast_type);
            let init_bool = let_stmt.initializer().is_some_and(is_bool_literal_expr);
            usize::from(ty_bool || init_bool)
        }
        ra_ap_syntax::ast::Pat::TuplePat(tuple_pat) => {
            count_tuple_let_mut_bool_flags(&tuple_pat, let_stmt.ty(), let_stmt.initializer())
        }
        _ => 0,
    }
}

/// Returns `true` iff `bin_expr` is a logical `&&` or `||` binary predicate expression.
/// Realises: [REQ-0608]
fn is_logical_and_or_bin_expr(bin_expr: &ra_ap_syntax::ast::BinExpr) -> bool {
    matches!(
        bin_expr.op_kind(),
        Some(ra_ap_syntax::ast::BinaryOp::LogicOp(_))
    )
}

/// Returns `true` when `bin_expr` is nested directly inside an enclosing logical `&&` / `||` predicate.
/// Realises: [REQ-0608]
fn is_child_of_logical_bin_expr(bin_expr: &ra_ap_syntax::ast::BinExpr) -> bool {
    let mut cursor = bin_expr.syntax().parent();
    let mut steps = 0usize;
    while let Some(parent) = cursor {
        if ra_ap_syntax::ast::ParenExpr::can_cast(parent.kind()) {
            cursor = parent.parent();
        } else if let Some(pref) = ra_ap_syntax::ast::PrefixExpr::cast(parent.clone()) {
            if pref.op_kind() == Some(ra_ap_syntax::ast::UnaryOp::Not) {
                cursor = parent.parent();
            } else {
                return false;
            }
        } else if let Some(parent_bin) = ra_ap_syntax::ast::BinExpr::cast(parent) {
            return is_logical_and_or_bin_expr(&parent_bin);
        } else {
            return false;
        }
        steps = steps.saturating_add(1);
        if steps > MAX_AST_DEPTH {
            return false;
        }
    }
    false
}

/// Strips enclosing parentheses and unary `!` wrappers from a candidate boolean predicate sub-expression.
/// Realises: [REQ-0608]
fn unwrap_logical_clause_wrappers(mut expr: ra_ap_syntax::ast::Expr) -> ra_ap_syntax::ast::Expr {
    let mut steps = 0usize;
    loop {
        match expr {
            ra_ap_syntax::ast::Expr::ParenExpr(paren) => {
                let Some(inner) = paren.expr() else {
                    return ra_ap_syntax::ast::Expr::ParenExpr(paren);
                };
                expr = inner;
            }
            ra_ap_syntax::ast::Expr::PrefixExpr(pref)
                if pref.op_kind() == Some(ra_ap_syntax::ast::UnaryOp::Not) =>
            {
                let Some(inner) = pref.expr() else {
                    return ra_ap_syntax::ast::Expr::PrefixExpr(pref);
                };
                expr = inner;
            }
            other => return other,
        }
        steps = steps.saturating_add(1);
        if steps > MAX_AST_DEPTH {
            return expr;
        }
    }
}

/// Iteratively counts atomic boolean clauses across a root logical `&&` / `||` `ast::BinExpr` tree.
/// Realises: [REQ-0608]
fn count_compound_predicate_clauses(
    bin_expr: &ra_ap_syntax::ast::BinExpr,
) -> Result<usize, CstMetricsError> {
    if !is_logical_and_or_bin_expr(bin_expr) || is_child_of_logical_bin_expr(bin_expr) {
        return Ok(0);
    }
    let start = u32::from(bin_expr.syntax().text_range().start()) as usize;
    let mut stack = Vec::new();
    if let Some(rhs) = bin_expr.rhs() {
        stack.push(rhs);
    }
    if let Some(lhs) = bin_expr.lhs() {
        stack.push(lhs);
    }
    let mut clauses = 0usize;
    while let Some(raw_expr) = stack.pop() {
        let expr = unwrap_logical_clause_wrappers(raw_expr);
        match expr {
            ra_ap_syntax::ast::Expr::BinExpr(inner_bin)
                if is_logical_and_or_bin_expr(&inner_bin) =>
            {
                if let Some(rhs) = inner_bin.rhs() {
                    stack.push(rhs);
                }
                if let Some(lhs) = inner_bin.lhs() {
                    stack.push(lhs);
                }
            }
            _ => {
                clauses = clauses
                    .checked_add(1)
                    .ok_or_else(|| depth_exhaustion_err(usize::MAX, MAX_AST_DEPTH, start))?;
            }
        }
    }
    Ok(clauses)
}

/// Records 0-based line indices covered by direct non-trivia tokens of `node` into `exec_lines`.
/// Realises: [REQ-0608]
fn record_callable_node_exec_lines(
    node: &ra_ap_syntax::SyntaxNode,
    line_starts: &[usize],
    exec_lines: &mut BTreeSet<usize>,
) -> Result<(), CstMetricsError> {
    if is_inside_doc_attr(node) {
        return Ok(());
    }
    for child in node.children_with_tokens() {
        let ra_ap_syntax::SyntaxElement::Token(tok) = child else {
            continue;
        };
        if is_trivia_token(&tok) {
            continue;
        }
        let range = tok.text_range();
        let start = u32::from(range.start()) as usize;
        let end = u32::from(range.end()) as usize;
        if start > end {
            return Err(CstMetricsError::LosslessRoundtripOrSpanMismatch {
                byte_start: start,
                byte_end: end,
            });
        }
        if let Some(last_byte) = end.checked_sub(1) {
            if start <= last_byte {
                let first_line = resolve_byte_to_line_index(line_starts, start)?;
                let last_line = resolve_byte_to_line_index(line_starts, last_byte)?;
                let mut line_cursor = first_line;
                while line_cursor <= last_line {
                    exec_lines.insert(line_cursor);
                    line_cursor = line_cursor
                        .checked_add(1)
                        .ok_or_else(|| depth_exhaustion_err(usize::MAX, MAX_AST_DEPTH, start))?;
                }
            }
        }
    }
    Ok(())
}

/// Accumulates `exec_lines`, `mut_bool_flag_count`, and `max_predicate_clauses` for one CST `node`.
/// Realises: [REQ-0608]
fn accumulate_callable_node_metrics(
    node: &ra_ap_syntax::SyntaxNode,
    is_root_fn: bool,
    line_starts: &[usize],
    acc: &mut CallableBodyAccumulator,
) -> Result<bool, CstMetricsError> {
    if !is_root_fn && (node_enters_test_scope(node) || ra_ap_syntax::ast::Fn::can_cast(node.kind()))
    {
        return Ok(false);
    }
    record_callable_node_exec_lines(node, line_starts, &mut acc.exec_lines)?;
    let start = u32::from(node.text_range().start()) as usize;
    if let Some(let_stmt) = ra_ap_syntax::ast::LetStmt::cast(node.clone()) {
        let delta = count_let_stmt_mut_bool_flags(&let_stmt);
        acc.mut_bool_flag_count = acc
            .mut_bool_flag_count
            .checked_add(delta)
            .ok_or_else(|| depth_exhaustion_err(usize::MAX, MAX_AST_DEPTH, start))?;
    }
    if let Some(bin_expr) = ra_ap_syntax::ast::BinExpr::cast(node.clone()) {
        let clauses = count_compound_predicate_clauses(&bin_expr)?;
        if clauses > acc.max_predicate_clauses {
            acc.max_predicate_clauses = clauses;
        }
    }
    Ok(true)
}

/// Extracts exact `FnAstMetrics` (`v2_metrics`) from a single `ra_ap_syntax::ast::Fn` CST node (`MT-US56-03`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C structural
/// complexity measurement rules (`extract_callable_v2_metrics`, MT-US56-03, US-56 / FEAT-42,
/// REQ-0608, FLS `fls_ikfvbeewame7`, FLS `fls_nsvzzbldhq53`). Computes per-callable executable
/// SLOC (`C_CST_01`), receiver-excluded parameter arity (`C_CST_02`), local `mut bool` flag count
/// (`C_CST_03`), compound boolean predicate clause arity, `#[rt_loop]` detection, and
/// `Result<T, String>` return-type detection without recursion.
///
/// ## 2. Invariants & Mathematical Properties
/// - Ordered Line Set: `exec_lines` is a `BTreeSet<usize>` of non-trivia, non-doc-attribute line
///   indices resolved via `line_starts.binary_search(...)` and `.get(...)` indexing (`exec_sloc == exec_lines.len()`).
/// - Receiver Exclusion: Strips `self`, `&self`, and `&mut self` (`ast::SelfParam`) from `param_count`.
/// - Bounded Iterative Walk: Enforces `MAX_AST_DEPTH = 64` via explicit heap worklist (`DEAP-E0804`).
/// - Complexity Budget Compliance: `2` formal parameters, `<= 48` executable SLOC, `0` mutable
///   `bool` flags, and compound boolean predicate arity `<= 2`.
///
/// ## 3. Preconditions
/// - `fn_node` borrows a lossless `ra_ap_syntax::ast::Fn` node within `MAX_AST_DEPTH = 64`.
/// - `line_starts` is a non-empty strictly increasing 0-based UTF-8 line-start byte offset slice.
///
/// ## 4. Postconditions
/// - Returns `Ok(FnAstMetrics)` populated with deterministic `BTreeSet` SLOC and `v2_metrics` fields.
/// - Returns `Err(CstMetricsError)` (`DEAP-E0802`..`DEAP-E0804`) fail-closed on span/line-table
///   mismatch, missing callable identifier, arithmetic overflow, or `MAX_AST_DEPTH = 64` breach.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(M log L) over M CST elements inside `fn_node` and L source lines.
/// - Space Complexity: O(M + L_fn) for the bounded iterative worklist stack and `BTreeSet<usize>`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Total function taking `2` parameters and returning typed `Result<FnAstMetrics, CstMetricsError>`.
/// - State: Pure read-only CST analysis with zero global or filesystem side effects.
/// - Action: Halts fail-closed on `DEAP-E0802`, `DEAP-E0803`, or `DEAP-E0804` without panicking.
/// - Resource: Explicit heap worklist bounded by `MAX_AST_DEPTH = 64` eliminates stack overflow.
///
/// ## 7. Realization & Traceability
/// Realises: [LosslessRustCstAnalyzer/extract_callable_v2_metrics] | [REQ-0608] | [FLS: fls_ikfvbeewame7] | [FLS: fls_nsvzzbldhq53] | SysML v2: part def LosslessRustCstAnalyzer
pub fn extract_callable_v2_metrics(
    fn_node: &ra_ap_syntax::ast::Fn,
    line_starts: &[usize],
) -> Result<FnAstMetrics, CstMetricsError> {
    let (symbol, byte_start, byte_end) = extract_fn_symbol_and_span(fn_node)?;
    let root_depth = compute_fn_root_cst_depth(fn_node)?;
    let (param_count, has_self_receiver) = extract_receiver_excluded_arity(fn_node)?;
    let is_rt_loop = has_outer_rt_loop_attr(fn_node);
    let returns_result_string_err = fn_returns_result_string_err(fn_node);
    let mut acc = CallableBodyAccumulator {
        exec_lines: BTreeSet::new(),
        mut_bool_flag_count: 0,
        max_predicate_clauses: 0,
    };
    let mut stack = vec![(fn_node.syntax().clone(), root_depth, true)];
    while let Some((node, depth, is_root_fn)) = stack.pop() {
        let start = u32::from(node.text_range().start()) as usize;
        if depth > MAX_AST_DEPTH {
            return Err(depth_exhaustion_err(depth, MAX_AST_DEPTH, start));
        }
        if accumulate_callable_node_metrics(&node, is_root_fn, line_starts, &mut acc)? {
            push_tier_worklist_children(&node, depth, MAX_AST_DEPTH, false, &mut stack)?;
        }
    }
    let exec_sloc = acc.exec_lines.len();
    Ok(FnAstMetrics {
        symbol,
        byte_start,
        byte_end,
        exec_lines: acc.exec_lines,
        exec_sloc,
        param_count,
        has_self_receiver,
        mut_bool_flag_count: acc.mut_bool_flag_count,
        max_predicate_clauses: acc.max_predicate_clauses,
        is_rt_loop,
        returns_result_string_err,
    })
}

/// Extracts and inserts `FnAstMetrics` into `callable_metrics` when `node` casts to `ast::Fn`.
/// Realises: [REQ-0608]
fn insert_callable_metrics_if_fn(
    node: &ra_ap_syntax::SyntaxNode,
    line_starts: &[usize],
    start_u32: u32,
    callable_metrics: &mut BTreeMap<(u32, String), FnAstMetrics>,
) -> Result<(), CstMetricsError> {
    let Some(fn_node) = ra_ap_syntax::ast::Fn::cast(node.clone()) else {
        return Ok(());
    };
    let metrics = extract_callable_v2_metrics(&fn_node, line_starts)?;
    let (byte_start, byte_end) = (metrics.byte_start, metrics.byte_end);
    let key = (start_u32, metrics.symbol.clone());
    if callable_metrics.insert(key, metrics).is_some() {
        return Err(CstMetricsError::LosslessRoundtripOrSpanMismatch {
            byte_start,
            byte_end,
        });
    }
    Ok(())
}

/// Assembles a sealed `DeapV2AstMetricsReport` from verified `DocTierInventory` and `callable_metrics`.
/// Realises: [REQ-0608]
fn build_v2_ast_metrics_report(
    cst: &VerifiedLosslessCst,
    doc_tier_inventory: DocTierInventory,
    callable_metrics: BTreeMap<(u32, String), FnAstMetrics>,
    observed_max_depth: usize,
) -> DeapV2AstMetricsReport {
    let production_exec_lines = doc_tier_inventory.production_exec_lines.clone();
    let production_exec_sloc = doc_tier_inventory.production_exec_sloc;
    let rust_tier_a_count = doc_tier_inventory.rust_tier_a_count;
    let rust_tier_bc_count = doc_tier_inventory.rust_tier_bc_count;
    let test_excluded_item_count = doc_tier_inventory.test_excluded_item_count;
    DeapV2AstMetricsReport {
        source_path: cst.source_path().to_string(),
        callable_metrics,
        doc_tier_inventory,
        production_exec_lines,
        production_exec_sloc,
        rust_tier_a_count,
        rust_tier_bc_count,
        test_excluded_item_count,
        observed_max_depth: cst.observed_max_depth().max(observed_max_depth),
        lossless_roundtrip_verified: true,
    }
}

/// Prunes `#[cfg(test)]`/`#[test]` subtrees and extracts `BTreeMap`-ordered `v2_metrics` (`MT-US56-03`).
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C structural
/// complexity and documentation tier governance (`analyze_cst_v2_metrics`, MT-US56-03, US-56 /
/// FEAT-42, REQ-0608, FLS `fls_ikfvbeewame7`, FLS `fls_nsvzzbldhq53`). Executes the
/// `AnalyzingMetricsAndTiers` pass over a sealed `VerifiedLosslessCst`, pruning `DocTier::TestExcluded`
/// (`#[cfg(test)]` and `#[test]`) subtrees before classifying `RustTierA`/`RustTierBC` items and
/// extracting per-callable `FnAstMetrics` into a deterministic `BTreeMap`.
///
/// ## 2. Invariants & Mathematical Properties
/// - Sealed Precondition Gate: Rejects unverified CST tokens (`!is_lossless_red_green_tree_ready()`)
///   fail-closed with `DEAP-E0801` (`FM-BP08-006`).
/// - Lossless Round-Trip & Depth Guard: Re-verifies `validate_metrics_thresholds` (`DEAP-E0803`),
///   root `rowan::GreenNode` byte/UTF-8 boundary parity (`DEAP-E0802`, `FM-BP08-008`), and iterative
///   `MAX_AST_DEPTH = 64` ceiling (`DEAP-E0804`, `FM-BP08-007`).
/// - Complexity Budget Compliance: `2` formal parameters, `<= 48` executable SLOC, `0` mutable
///   `bool` flags, and compound boolean predicate arity `<= 2`.
///
/// ## 3. Preconditions
/// - `cst` borrows a sealed `VerifiedLosslessCst` produced by `parse_verified_lossless_cst`.
/// - `thresholds` satisfies `validate_metrics_thresholds` (`C_CST_01`..`C_CST_03`, `MAX_AST_DEPTH = 64`).
///
/// ## 4. Postconditions
/// - Returns `Ok(DeapV2AstMetricsReport)` ordered in `BTreeMap<(u32, String), FnAstMetrics>` with
///   100% of `#[cfg(test)]` and `#[test]` subtrees excluded from production metrics and SLOC.
/// - Returns `Err(CstMetricsError)` (`DEAP-E0801`..`DEAP-E0804`) fail-closed on any invariant breach.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N log L + F log F) over N CST elements, L source lines, and F callables.
/// - Space Complexity: O(N + F + L) for the bounded iterative worklist stack and `BTreeMap`/`BTreeSet`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Total function taking `2` parameters and returning typed `Result<DeapV2AstMetricsReport, CstMetricsError>`.
/// - State: Pure read-only CST analysis performing zero global or filesystem mutations.
/// - Action: Suppresses partial `DeapV2AstMetricsReport` emission on `DEAP-E0801`..`DEAP-E0804` faults.
/// - Resource: Explicit heap worklist `(SyntaxNode, usize, bool)` bounded by `MAX_AST_DEPTH = 64`.
///
/// ## 7. Realization & Traceability
/// Realises: [LosslessRustCstAnalyzer/analyze_cst_v2_metrics] | [REQ-0608] | [FLS: fls_ikfvbeewame7] | [FLS: fls_nsvzzbldhq53] | SysML v2: part def LosslessRustCstAnalyzer
pub fn analyze_cst_v2_metrics(
    cst: &VerifiedLosslessCst,
    thresholds: &CstMetricsThresholds,
) -> Result<DeapV2AstMetricsReport, CstMetricsError> {
    let doc_tier_inventory = classify_cfg_test_excluded_doc_tier(cst, thresholds, false)?;
    let mut callable_metrics: BTreeMap<(u32, String), FnAstMetrics> = BTreeMap::new();
    let mut stack = vec![(cst.syntax_node().clone(), 0usize, false)];
    let mut observed_max_depth = 0usize;
    while let Some((node, depth, inside_test_scope)) = stack.pop() {
        let range = node.text_range();
        let start_u32 = u32::from(range.start());
        let (start, end) = (start_u32 as usize, u32::from(range.end()) as usize);
        if depth > thresholds.max_ast_depth {
            return Err(depth_exhaustion_err(depth, thresholds.max_ast_depth, start));
        }
        verify_utf8_char_boundary_span(cst.source_text(), start, end)?;
        observed_max_depth = observed_max_depth.max(depth);
        if !(inside_test_scope || node_enters_test_scope(&node)) {
            insert_callable_metrics_if_fn(
                &node,
                cst.line_starts(),
                start_u32,
                &mut callable_metrics,
            )?;
            push_tier_worklist_children(&node, depth, thresholds.max_ast_depth, false, &mut stack)?;
        }
    }
    Ok(build_v2_ast_metrics_report(
        cst,
        doc_tier_inventory,
        callable_metrics,
        observed_max_depth,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

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

    /// Builds an in-memory pinned CST `LockManifestSnapshot` fixture with configurable vendored DAG depth.
    /// Realises: [REQ-0608]
    fn build_cst_envelope_fixture_snapshot(
        chain_depth: usize,
    ) -> crate::workspace::LockManifestSnapshot {
        use crate::workspace::{LockManifestSnapshot, LockedPackageRecord, PackageSourceKind};
        use std::collections::BTreeMap;

        let valid_sha =
            "14b574c58582fa59fa43a2feb6608b8744184659f08a2e0117e4b8224d95ed61".to_string();
        let mut packages = BTreeMap::new();
        let mut root_deps = BTreeSet::new();
        for (name, ver) in [
            ("ra_ap_syntax", "0.0.360"),
            ("rowan", "0.17.0"),
            ("syn", "3.0.6"),
            ("quote", "1.0.47"),
            ("prettyplease", "0.3.0"),
        ] {
            let key = format!("{name}@{ver}");
            root_deps.insert(key.clone());
            packages.insert(
                key,
                LockedPackageRecord {
                    name: name.to_string(),
                    version: ver.to_string(),
                    checksum: Some(valid_sha.clone()),
                    source_kind: PackageSourceKind::VendoredBundle,
                    dependencies: BTreeSet::new(),
                },
            );
        }
        packages.insert(
            "demo@0.1.0".to_string(),
            LockedPackageRecord {
                name: "demo".to_string(),
                version: "0.1.0".to_string(),
                checksum: None,
                source_kind: PackageSourceKind::WorkspacePath,
                dependencies: root_deps,
            },
        );
        for idx in 0..chain_depth {
            let cur_name = if idx == 0 {
                "demo".to_string()
            } else {
                format!("chain_{idx}")
            };
            let cur_key = format!("{cur_name}@0.1.0");
            let next_name = format!("chain_{}", idx.saturating_add(1));
            let next_key = format!("{next_name}@0.1.0");
            packages.insert(
                next_key.clone(),
                LockedPackageRecord {
                    name: next_name,
                    version: "0.1.0".to_string(),
                    checksum: Some(valid_sha.clone()),
                    source_kind: PackageSourceKind::VendoredBundle,
                    dependencies: BTreeSet::new(),
                },
            );
            if let Some(cur_rec) = packages.get_mut(&cur_key) {
                cur_rec.dependencies.insert(next_key);
            }
        }
        LockManifestSnapshot {
            cargo_toml_sha256: [0x33u8; 32],
            cargo_lock_sha256: [0x44u8; 32],
            packages,
            workspace_members: BTreeSet::from(["demo".to_string()]),
            offline_locked_enforced: true,
            vendor_bundle_supported: true,
        }
    }

    /// Verifies C_CST_01..C_CST_06 envelope sealing, deterministic SHA-256 receipts, and fail-closed drift rejection.
    /// Realises: [REQ-0608]
    #[test]
    fn seals_supply_chain_receipt_and_rejects_envelope_drift(
    ) -> Result<(), Box<dyn std::error::Error>> {
        use crate::workspace::{
            parse_locked_manifest_snapshot, verify_pinned_cst_dependency_graph,
            LockedPackageRecord, OfflineLockGovernanceError, PackageSourceKind, MAX_AST_DEPTH,
            MAX_CARGO_LOCK_BYTES,
        };

        let required_cst = BTreeSet::from([
            "ra_ap_syntax".to_string(),
            "rowan".to_string(),
            "syn".to_string(),
            "quote".to_string(),
            "prettyplease".to_string(),
        ]);
        let forbidden_cst = BTreeSet::from([
            "ra_ap_hir".to_string(),
            "ra_ap_ide".to_string(),
            "rustc_private".to_string(),
        ]);

        // 1. Nominal live workspace snapshot sealing (AC-US-51-01)
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .ok_or("missing workspace root")?;
        let live_toml = fs::read(repo_root.join("Cargo.toml"))?;
        let live_lock = fs::read(repo_root.join("Cargo.lock"))?;
        let live_snap =
            parse_locked_manifest_snapshot(&live_toml, &live_lock, None, MAX_CARGO_LOCK_BYTES)?;
        let live_verified = verify_pinned_cst_dependency_graph(
            &live_snap,
            &required_cst,
            &forbidden_cst,
            MAX_AST_DEPTH,
        )?;
        let nominal_env = CstComplexityEnvelope::default();
        let receipt_a =
            verify_analyzer_envelope_and_seal_receipt(&live_snap, &live_verified, &nominal_env)?;
        let receipt_b =
            verify_analyzer_envelope_and_seal_receipt(&live_snap, &live_verified, &nominal_env)?;
        assert!(receipt_a.supply_chain_receipt_sealed);
        assert_eq!(receipt_a, receipt_b);
        assert_ne!(receipt_a.seal_sha256, [0u8; 32]);
        assert_eq!(receipt_a.cargo_toml_sha256, live_snap.cargo_toml_sha256);
        assert_eq!(receipt_a.cargo_lock_sha256, live_snap.cargo_lock_sha256);
        assert_eq!(receipt_a.verified_crates, live_verified);
        assert_eq!(receipt_a.workspace_members.len(), 6);
        assert_eq!(receipt_a.verify_snapshot_parity(&live_snap), Ok(()));

        // 2. Boundary MAX_AST_DEPTH == 64 vendored bundle & C_CST_01..C_CST_06 bounds (AC-US-51-02)
        let depth64_snap = build_cst_envelope_fixture_snapshot(MAX_AST_DEPTH);
        let depth64_verified = verify_pinned_cst_dependency_graph(
            &depth64_snap,
            &required_cst,
            &forbidden_cst,
            MAX_AST_DEPTH,
        )?;
        let upper_receipt: SupplyChainVerificationReceipt =
            verify_analyzer_envelope_and_seal_receipt(
                &depth64_snap,
                &depth64_verified,
                &nominal_env,
            )?;
        let lower_env = CstComplexityEnvelope {
            max_fn_exec_sloc: 1,
            max_fn_param_count: 0,
            ..nominal_env.clone()
        };
        let lower_receipt = verify_analyzer_envelope_and_seal_receipt(
            &depth64_snap,
            &depth64_verified,
            &lower_env,
        )?;
        assert!(upper_receipt.supply_chain_receipt_sealed);
        assert!(lower_receipt.supply_chain_receipt_sealed);
        assert_ne!(upper_receipt.seal_sha256, lower_receipt.seal_sha256);

        // 3. Fail-closed C_CST_01..C_CST_06 & envelope invariant drift -> DEAP-E0803 (AC-US-51-05)
        let bad_envelopes = [
            CstComplexityEnvelope {
                max_fn_exec_sloc: 0,
                ..nominal_env.clone()
            },
            CstComplexityEnvelope {
                max_fn_exec_sloc: 51,
                ..nominal_env.clone()
            },
            CstComplexityEnvelope {
                max_fn_param_count: 6,
                ..nominal_env.clone()
            },
            CstComplexityEnvelope {
                max_mut_bool_flags: 1,
                ..nominal_env.clone()
            },
            CstComplexityEnvelope {
                stage_fixed_point_mismatch_files: 1,
                ..nominal_env.clone()
            },
            CstComplexityEnvelope {
                runtime_llm_token_count: 1,
                ..nominal_env.clone()
            },
            CstComplexityEnvelope {
                tier_bc_max_doc_lines: 1,
                ..nominal_env.clone()
            },
            CstComplexityEnvelope {
                tier_bc_max_doc_lines: 3,
                ..nominal_env.clone()
            },
            CstComplexityEnvelope {
                max_predicate_arity: 2,
                ..nominal_env.clone()
            },
            CstComplexityEnvelope {
                max_predicate_arity: 4,
                ..nominal_env.clone()
            },
            CstComplexityEnvelope {
                lossless_roundtrip_required: false,
                ..nominal_env.clone()
            },
            CstComplexityEnvelope {
                deterministic_uuid: "00000000-0000-0000-0000-000000000000".to_string(),
                ..nominal_env.clone()
            },
        ];
        for drifted_env in &bad_envelopes {
            let err =
                verify_analyzer_envelope_and_seal_receipt(&live_snap, &live_verified, drifted_env);
            assert_eq!(
                err,
                Err(OfflineLockGovernanceError::CstConstraintEnvelopeViolation)
            );
            assert_eq!(
                OfflineLockGovernanceError::CstConstraintEnvelopeViolation.diagnostic_code(),
                "DEAP-E0803"
            );
        }

        // 4. Fail-closed policy, pre-receipt out-of-order, TOCTOU drift, forbidden crate & depth > 64 -> DEAP-E0804
        let base_snap = build_cst_envelope_fixture_snapshot(1);
        let base_verified = verify_pinned_cst_dependency_graph(
            &base_snap,
            &required_cst,
            &forbidden_cst,
            MAX_AST_DEPTH,
        )?;

        let mut policy_off = base_snap.clone();
        policy_off.offline_locked_enforced = false;
        assert_eq!(
            verify_analyzer_envelope_and_seal_receipt(&policy_off, &base_verified, &nominal_env),
            Err(OfflineLockGovernanceError::LockPolicyDisabled)
        );

        assert_eq!(
            verify_analyzer_envelope_and_seal_receipt(&base_snap, &BTreeSet::new(), &nominal_env),
            Err(OfflineLockGovernanceError::ManifestToLockDriftDetected)
        );

        let mut drifted_verified = base_verified.clone();
        drifted_verified.remove("demo@0.1.0");
        assert_eq!(
            verify_analyzer_envelope_and_seal_receipt(&base_snap, &drifted_verified, &nominal_env),
            Err(OfflineLockGovernanceError::ManifestToLockDriftDetected)
        );

        let mut drifted_digest_snap = live_snap.clone();
        drifted_digest_snap.cargo_lock_sha256 = [0xFFu8; 32];
        assert_eq!(
            receipt_a.verify_snapshot_parity(&drifted_digest_snap),
            Err(OfflineLockGovernanceError::ManifestToLockDriftDetected)
        );

        let mut tampered_receipt = receipt_a.clone();
        tampered_receipt.seal_sha256 = [0xAAu8; 32];
        assert_eq!(
            tampered_receipt.verify_snapshot_parity(&live_snap),
            Err(OfflineLockGovernanceError::ManifestToLockDriftDetected)
        );

        let mut hir_snap = base_snap.clone();
        hir_snap.packages.insert(
            "ra_ap_hir@0.0.360".to_string(),
            LockedPackageRecord {
                name: "ra_ap_hir".to_string(),
                version: "0.0.360".to_string(),
                checksum: Some(
                    "14b574c58582fa59fa43a2feb6608b8744184659f08a2e0117e4b8224d95ed61".to_string(),
                ),
                source_kind: PackageSourceKind::VendoredBundle,
                dependencies: BTreeSet::new(),
            },
        );
        let mut hir_verified = base_verified.clone();
        hir_verified.insert("ra_ap_hir@0.0.360".to_string());
        assert_eq!(
            verify_analyzer_envelope_and_seal_receipt(&hir_snap, &hir_verified, &nominal_env),
            Err(OfflineLockGovernanceError::ForbiddenSemanticDependencyDetected)
        );

        let mut missing_cst_snap = base_snap;
        missing_cst_snap.packages.remove("ra_ap_syntax@0.0.360");
        if let Some(demo) = missing_cst_snap.packages.get_mut("demo@0.1.0") {
            demo.dependencies.remove("ra_ap_syntax@0.0.360");
        }
        let mut missing_cst_verified = base_verified;
        missing_cst_verified.remove("ra_ap_syntax@0.0.360");
        assert_eq!(
            verify_analyzer_envelope_and_seal_receipt(
                &missing_cst_snap,
                &missing_cst_verified,
                &nominal_env
            ),
            Err(OfflineLockGovernanceError::MissingRequiredCstDependency)
        );

        let depth65_snap = build_cst_envelope_fixture_snapshot(MAX_AST_DEPTH.saturating_add(1));
        let depth65_keys: BTreeSet<String> = depth65_snap.packages.keys().cloned().collect();
        assert_eq!(
            verify_analyzer_envelope_and_seal_receipt(&depth65_snap, &depth65_keys, &nominal_env),
            Err(OfflineLockGovernanceError::PackageGraphDepthOrCycleExceeded)
        );

        Ok(())
    }

    /// Verifies C_CST_01..C_CST_03 threshold envelope bounds and typed DEAP-E0801..DEAP-E0804 error taxonomy.
    /// Realises: [REQ-0608]
    #[test]
    fn rejects_out_of_bounds_cst_metrics_thresholds() -> Result<(), Box<dyn std::error::Error>> {
        let nominal = crate::CstMetricsThresholds::default();
        assert_eq!(nominal.max_fn_exec_sloc, 50);
        assert_eq!(nominal.max_fn_param_count, 5);
        assert_eq!(nominal.max_mut_bool_flags, 0);
        assert_eq!(nominal.max_predicate_arity, 3);
        assert_eq!(nominal.max_ast_depth, MAX_AST_DEPTH);
        assert!(nominal.lossless_roundtrip_required);
        assert_eq!(
            nominal.deterministic_uuid,
            "6ba7b810-9dad-11d1-80b4-00c04fd430c8"
        );
        assert_eq!(crate::validate_metrics_thresholds(&nominal), Ok(()));

        let lower_valid = CstMetricsThresholds {
            max_fn_exec_sloc: 1,
            max_fn_param_count: 0,
            ..nominal
        };
        assert_eq!(validate_metrics_thresholds(&lower_valid), Ok(()));

        let mid_valid = CstMetricsThresholds {
            max_fn_exec_sloc: 45,
            max_fn_param_count: 3,
            ..nominal
        };
        assert_eq!(validate_metrics_thresholds(&mid_valid), Ok(()));

        let invalid_cases: [(CstMetricsThresholds, &'static str, &'static str); 13] = [
            (
                CstMetricsThresholds {
                    max_fn_exec_sloc: 0,
                    ..nominal
                },
                "C_CST_01",
                "max_fn_exec_sloc",
            ),
            (
                CstMetricsThresholds {
                    max_fn_exec_sloc: 51,
                    ..nominal
                },
                "C_CST_01",
                "max_fn_exec_sloc",
            ),
            (
                CstMetricsThresholds {
                    max_fn_exec_sloc: usize::MAX,
                    ..nominal
                },
                "C_CST_01",
                "max_fn_exec_sloc",
            ),
            (
                CstMetricsThresholds {
                    max_fn_param_count: 6,
                    ..nominal
                },
                "C_CST_02",
                "max_fn_param_count",
            ),
            (
                CstMetricsThresholds {
                    max_mut_bool_flags: 1,
                    ..nominal
                },
                "C_CST_03",
                "max_mut_bool_flags",
            ),
            (
                CstMetricsThresholds {
                    max_predicate_arity: 2,
                    ..nominal
                },
                "C_CST_PREDICATE_ARITY",
                "max_predicate_arity",
            ),
            (
                CstMetricsThresholds {
                    max_predicate_arity: 4,
                    ..nominal
                },
                "C_CST_PREDICATE_ARITY",
                "max_predicate_arity",
            ),
            (
                CstMetricsThresholds {
                    max_ast_depth: 0,
                    ..nominal
                },
                "C_CST_MAX_AST_DEPTH",
                "max_ast_depth",
            ),
            (
                CstMetricsThresholds {
                    max_ast_depth: 63,
                    ..nominal
                },
                "C_CST_MAX_AST_DEPTH",
                "max_ast_depth",
            ),
            (
                CstMetricsThresholds {
                    max_ast_depth: 65,
                    ..nominal
                },
                "C_CST_MAX_AST_DEPTH",
                "max_ast_depth",
            ),
            (
                CstMetricsThresholds {
                    lossless_roundtrip_required: false,
                    ..nominal
                },
                "C_CST_LOSSLESS_ROUNDTRIP",
                "lossless_roundtrip_required",
            ),
            (
                CstMetricsThresholds {
                    deterministic_uuid: "",
                    ..nominal
                },
                "C_CST_DETERMINISTIC_UUID",
                "deterministic_uuid",
            ),
            (
                CstMetricsThresholds {
                    deterministic_uuid: "00000000-0000-0000-0000-000000000000",
                    ..nominal
                },
                "C_CST_DETERMINISTIC_UUID",
                "deterministic_uuid",
            ),
        ];

        for (mutated, expected_code, expected_symbol) in &invalid_cases {
            let err = match validate_metrics_thresholds(mutated) {
                Ok(()) => return Err(format!("expected Err for {expected_symbol}").into()),
                Err(err) => err,
            };
            assert_eq!(err.diagnostic_code(), "DEAP-E0803");
            assert_eq!(err.byte_span(), (0, 0));
            assert_eq!(
                err,
                crate::CstMetricsError::ThresholdEnvelopeOrMetricsViolation {
                    code: expected_code,
                    symbol: (*expected_symbol).to_string(),
                    byte_start: 0,
                    byte_end: 0,
                }
            );
        }

        let taxonomy_samples: [(CstMetricsError, &'static str, (usize, usize), bool); 5] = [
            (
                CstMetricsError::UnverifiedParsePrecondition { state: "Idle" },
                "DEAP-E0801",
                (0, 0),
                false,
            ),
            (
                CstMetricsError::LosslessRoundtripOrSpanMismatch {
                    byte_start: 12,
                    byte_end: 24,
                },
                "DEAP-E0802",
                (12, 24),
                true,
            ),
            (
                CstMetricsError::MalformedUtf8OrRustSyntax {
                    byte_start: 4,
                    byte_end: 8,
                    detail: "unexpected EOF".to_string(),
                },
                "DEAP-E0803",
                (4, 8),
                true,
            ),
            (
                CstMetricsError::ThresholdEnvelopeOrMetricsViolation {
                    code: "C_CST_01",
                    symbol: "max_fn_exec_sloc".to_string(),
                    byte_start: 0,
                    byte_end: 0,
                },
                "DEAP-E0803",
                (0, 0),
                false,
            ),
            (
                CstMetricsError::AstDepthOrArithmeticExhaustion {
                    observed_depth: 65,
                    max_depth: MAX_AST_DEPTH,
                    byte_offset: 128,
                },
                "DEAP-E0804",
                (128, 128),
                false,
            ),
        ];

        for (sample, expected_diag_code, expected_span, is_syntax_category) in &taxonomy_samples {
            assert_eq!(sample.diagnostic_code(), *expected_diag_code);
            assert_eq!(sample.byte_span(), *expected_span);
            let formatted = sample.to_string();
            assert!(formatted.contains(expected_diag_code));
            assert!(!formatted.as_bytes().contains(&0x0Au8));
            assert!(!formatted.as_bytes().contains(&0x0Du8));
            assert!(!formatted.as_bytes().contains(&0x1Bu8));
            let bridged = CoreError::from(sample.clone());
            assert!(bridged.as_str().contains(expected_diag_code));
            if *is_syntax_category {
                assert!(matches!(bridged, CoreError::Syntax { .. }));
            } else {
                assert!(matches!(bridged, CoreError::Validation { .. }));
            }
        }

        let control_injected = CstMetricsError::MalformedUtf8OrRustSyntax {
            byte_start: 1,
            byte_end: 5,
            detail: "\x1b[31mline1\nline2\r".to_string(),
        };
        let sanitized_msg = control_injected.to_string();
        assert!(!sanitized_msg.as_bytes().contains(&0x0Au8));
        assert!(!sanitized_msg.as_bytes().contains(&0x0Du8));
        assert!(!sanitized_msg.as_bytes().contains(&0x1Bu8));

        Ok(())
    }

    /// Synthesizes a valid Rust function wrapping a literal inside `paren_layers` nested parentheses.
    /// Realises: [REQ-0608]
    fn build_nested_paren_rust_fixture(paren_layers: usize) -> String {
        let mut out = String::from("pub fn depth_probe() -> u32 { ");
        for _ in 0..paren_layers {
            out.push('(');
        }
        out.push_str("1u32");
        for _ in 0..paren_layers {
            out.push(')');
        }
        out.push_str(" }\n");
        out
    }

    /// Verifies lossless Rowan GreenNode roundtrip parity, UTF-8 char boundaries, and iterative MAX_AST_DEPTH = 64 guard.
    /// Realises: [REQ-0608]
    #[test]
    fn enforces_lossless_green_node_roundtrip_and_max_depth_64(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let thresholds = crate::CstMetricsThresholds::default();
        let nominal_src = concat!(
            "//! Module doc with Greek \u{03B1}\u{03B2} and math \u{2211}.\n",
            "\n",
            "/// Computes deterministic bounded sum.\n",
            "/// Realises: [REQ-0608]\n",
            "pub fn sum_pair(lhs: u32, rhs: u32) -> u32 {\n",
            "    let raw_note = r#\"unbalanced { {{ braces inside raw string\"#;\n",
            "    let _ = raw_note.len();\n",
            "    lhs.saturating_add(rhs)\n",
            "}\n",
        );

        let verified: crate::VerifiedLosslessCst = crate::parse_verified_lossless_cst(
            "src/nominal_fixture.rs",
            nominal_src.as_bytes(),
            &thresholds,
        )?;
        assert!(verified.is_lossless_red_green_tree_ready());
        assert_eq!(verified.source_path(), "src/nominal_fixture.rs");
        assert_eq!(verified.source_text(), nominal_src);
        assert_eq!(verified.source_bytes(), nominal_src.as_bytes());
        assert_eq!(
            verified.green_node().to_string().as_bytes(),
            nominal_src.as_bytes()
        );
        assert_eq!(
            verified.syntax_node().text().to_string().as_bytes(),
            nominal_src.as_bytes()
        );
        assert!(verified.observed_max_depth() > 0);
        assert!(verified.observed_max_depth() <= MAX_AST_DEPTH);
        assert!(verified.verified_node_count() > 0);
        assert_eq!(verified.line_starts().first(), Some(&0usize));
        assert_eq!(verified.clone(), verified);
        assert_eq!(
            ra_ap_syntax::AstNode::syntax(verified.source_file())
                .text()
                .to_string(),
            nominal_src
        );

        // Auto-cleaned temp-dir filesystem round-trip parity.
        let temp_dir =
            std::env::temp_dir().join(format!(".deap_td05_lossless_cst_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir)?;
        let temp_rs = temp_dir.join("fixture.rs");
        std::fs::write(&temp_rs, nominal_src.as_bytes())?;
        let disk_bytes = std::fs::read(&temp_rs);
        let _ = std::fs::remove_dir_all(&temp_dir);
        let disk_bytes = disk_bytes?;
        let disk_cst = crate::parse_verified_lossless_cst("fixture.rs", &disk_bytes, &thresholds)?;
        assert_eq!(disk_cst.source_bytes(), nominal_src.as_bytes());

        let empty_cst = crate::parse_verified_lossless_cst("src/empty.rs", b"", &thresholds)?;
        assert_eq!(empty_cst.observed_max_depth(), 0);
        assert_eq!(empty_cst.source_bytes(), b"");

        // Calibrate exact SyntaxNode nesting depth to test MAX_AST_DEPTH == 64 boundary and 65 breach.
        let seed_layers = 8usize;
        let base_src = build_nested_paren_rust_fixture(seed_layers);
        let base_cst =
            crate::parse_verified_lossless_cst("src/base.rs", base_src.as_bytes(), &thresholds)?;
        let base_depth = base_cst.observed_max_depth();
        assert!(base_depth < MAX_AST_DEPTH);
        let exact_layers = seed_layers.saturating_add(MAX_AST_DEPTH.saturating_sub(base_depth));

        let depth_64_src = build_nested_paren_rust_fixture(exact_layers);
        let depth_64_cst = crate::parse_verified_lossless_cst(
            "src/depth_64.rs",
            depth_64_src.as_bytes(),
            &thresholds,
        )?;
        assert_eq!(depth_64_cst.observed_max_depth(), MAX_AST_DEPTH);
        assert_eq!(
            depth_64_cst.green_node().to_string().as_bytes(),
            depth_64_src.as_bytes()
        );

        let depth_65_src = build_nested_paren_rust_fixture(exact_layers.saturating_add(1));
        let depth_65_err = match crate::parse_verified_lossless_cst(
            "src/depth_65.rs",
            depth_65_src.as_bytes(),
            &thresholds,
        ) {
            Ok(_) => return Err("expected depth 65 CST to fail closed with DEAP-E0804".into()),
            Err(err) => err,
        };
        assert_eq!(depth_65_err.diagnostic_code(), "DEAP-E0804");
        assert!(matches!(
            depth_65_err,
            CstMetricsError::AstDepthOrArithmeticExhaustion {
                observed_depth: 65,
                max_depth: MAX_AST_DEPTH,
                ..
            }
        ));

        // Fail-closed invalid UTF-8 byte sequence -> DEAP-E0803.
        let mut bad_utf8 = b"pub fn ok() {}\n".to_vec();
        bad_utf8.insert(7, 0xFF);
        let utf8_err =
            match crate::parse_verified_lossless_cst("src/bad_utf8.rs", &bad_utf8, &thresholds) {
                Ok(_) => return Err("expected invalid UTF-8 to fail closed".into()),
                Err(err) => err,
            };
        assert_eq!(utf8_err.diagnostic_code(), "DEAP-E0803");
        assert!(matches!(
            utf8_err,
            CstMetricsError::MalformedUtf8OrRustSyntax {
                byte_start: 7,
                byte_end: 8,
                ..
            }
        ));

        // Fail-closed malformed Rust syntax -> DEAP-E0803.
        for malformed in [
            b"pub fn incomplete() -> u32 { 1 + }".as_slice(),
            b"pub fn unclosed(".as_slice(),
        ] {
            let syn_err = match crate::parse_verified_lossless_cst(
                "src/malformed.rs",
                malformed,
                &thresholds,
            ) {
                Ok(_) => return Err("expected malformed Rust syntax to fail closed".into()),
                Err(err) => err,
            };
            assert_eq!(syn_err.diagnostic_code(), "DEAP-E0803");
            assert!(matches!(
                syn_err,
                CstMetricsError::MalformedUtf8OrRustSyntax { .. }
            ));
        }

        // Fail-closed mutated thresholds -> DEAP-E0803.
        let bad_thresholds = CstMetricsThresholds {
            max_fn_exec_sloc: 51,
            ..thresholds
        };
        let thr_err = match crate::parse_verified_lossless_cst(
            "src/valid.rs",
            b"pub fn valid() {}\n",
            &bad_thresholds,
        ) {
            Ok(_) => return Err("expected mutated thresholds to fail closed".into()),
            Err(err) => err,
        };
        assert_eq!(thr_err.diagnostic_code(), "DEAP-E0803");
        assert!(matches!(
            thr_err,
            CstMetricsError::ThresholdEnvelopeOrMetricsViolation {
                code: "C_CST_01",
                ..
            }
        ));

        // Fault-injection: GreenNode byte divergence & mid-codepoint UTF-8 span -> DEAP-E0802.
        let alpha_cst = crate::parse_verified_lossless_cst(
            "src/alpha.rs",
            b"pub fn alpha() {}\n",
            &thresholds,
        )?;
        let mismatch_err = match verify_green_tree_roundtrip_and_depth(
            alpha_cst.green_node(),
            "pub fn bravo() {}\n",
            MAX_AST_DEPTH,
        ) {
            Ok(_) => return Err("expected GreenNode divergence to fail closed".into()),
            Err(err) => err,
        };
        assert_eq!(mismatch_err.diagnostic_code(), "DEAP-E0802");
        assert_eq!(
            mismatch_err,
            CstMetricsError::LosslessRoundtripOrSpanMismatch {
                byte_start: 7,
                byte_end: 12,
            }
        );

        let len_mismatch_err = match verify_green_tree_roundtrip_and_depth(
            alpha_cst.green_node(),
            "pub fn alpha() {}\n// extra",
            MAX_AST_DEPTH,
        ) {
            Ok(_) => return Err("expected trailing length mismatch to fail closed".into()),
            Err(err) => err,
        };
        assert_eq!(len_mismatch_err.diagnostic_code(), "DEAP-E0802");

        let multi_utf8 = "// \u{03B1}\u{03B2}\npub fn f() {}\n";
        assert!(!multi_utf8.is_char_boundary(4));
        let boundary_err = match verify_utf8_char_boundary_span(multi_utf8, 3, 4) {
            Ok(()) => return Err("expected mid-codepoint span to fail closed".into()),
            Err(err) => err,
        };
        assert_eq!(boundary_err.diagnostic_code(), "DEAP-E0802");
        assert_eq!(
            boundary_err,
            CstMetricsError::LosslessRoundtripOrSpanMismatch {
                byte_start: 3,
                byte_end: 4,
            }
        );

        let ascii_let_cst = crate::parse_verified_lossless_cst(
            "src/ascii_let.rs",
            b"pub fn g() { let x = 1; }\n",
            &thresholds,
        )?;
        let mid_token_err = match verify_green_tree_roundtrip_and_depth(
            ascii_let_cst.green_node(),
            "pub fn g() { let \u{03B1} = 1; }\n",
            MAX_AST_DEPTH,
        ) {
            Ok(_) => return Err("expected mid-codepoint GreenNode token boundary trap".into()),
            Err(err) => err,
        };
        assert_eq!(mid_token_err.diagnostic_code(), "DEAP-E0802");
        assert_eq!(
            mid_token_err,
            CstMetricsError::LosslessRoundtripOrSpanMismatch {
                byte_start: 17,
                byte_end: 18,
            }
        );

        // Fault-injection: checked depth increment arithmetic overflow -> DEAP-E0804.
        let overflow_err = match checked_next_cst_depth(usize::MAX, MAX_AST_DEPTH, 96) {
            Ok(_) => return Err("expected usize::MAX depth increment to fail closed".into()),
            Err(err) => err,
        };
        assert_eq!(overflow_err.diagnostic_code(), "DEAP-E0804");
        assert_eq!(
            overflow_err,
            CstMetricsError::AstDepthOrArithmeticExhaustion {
                observed_depth: usize::MAX,
                max_depth: MAX_AST_DEPTH,
                byte_offset: 96,
            }
        );

        Ok(())
    }

    /// Synthesizes a 70-SLOC test helper function body inside or outside a `#[cfg(test)]` block.
    /// Realises: [REQ-0608]
    fn build_seventy_sloc_test_helper_fn() -> String {
        let mut out = String::from("    fn build_70_sloc_fixture() -> u32 {\n");
        out.push_str("        let mut acc = 0u32;\n");
        for idx in 0..68u32 {
            out.push_str(&format!("        acc = acc.saturating_add({idx}u32);\n"));
        }
        out.push_str("        acc\n    }\n");
        out
    }

    /// Verifies subtree-scoped `#[cfg(test)]` and `#[test]` pruning, `DocTier` classification, and fail-closed `DEAP-E0801`..`DEAP-E0804` traps.
    /// Realises: [REQ-0608]
    #[test]
    fn prunes_cfg_test_subtrees_from_production_doc_tiers() -> Result<(), Box<dyn std::error::Error>>
    {
        let thresholds = crate::CstMetricsThresholds::default();

        // 1. DocTier taxonomy variants, labels, and production predicate.
        assert!(crate::DocTier::RustTierA.is_production());
        assert!(crate::DocTier::RustTierBC.is_production());
        assert!(!crate::DocTier::TestExcluded.is_production());
        assert_eq!(crate::DocTier::RustTierA.label(), "RustTierA");
        assert_eq!(crate::DocTier::RustTierBC.label(), "RustTierBC");
        assert_eq!(crate::DocTier::TestExcluded.label(), "TestExcluded");

        // 2. Interleaved cfg(test) module (with 70-SLOC helper), #[test] fn, and preceding/trailing production items.
        let mut interleaved = String::new();
        interleaved.push_str("//! Module header doc.\n\n");
        interleaved.push_str("/// Primary sensor record.\n/// Realises: [REQ-0608]\n");
        interleaved.push_str("pub struct AlphaSensor {\n    pub reading: u32,\n}\n\n");
        interleaved.push_str("/// Preceding public reader.\n/// Realises: [REQ-0608]\n");
        interleaved.push_str("pub fn read_before(sample: u32) -> u32 {\n    sample\n}\n\n");
        interleaved.push_str("#[cfg(test)]\nmod tests {\n");
        interleaved.push_str("    pub struct MockHarness;\n");
        interleaved.push_str(&build_seventy_sloc_test_helper_fn());
        interleaved.push_str(
            "    #[test]\n    fn verifies_harness() {\n        let _ = build_70_sloc_fixture();\n    }\n",
        );
        interleaved.push_str("}\n\n");
        interleaved.push_str("#[test]\nfn top_level_unit_test() {\n    let x = 42u32;\n    assert_eq!(x, 42u32);\n}\n\n");
        interleaved.push_str(
            "#[cfg( test )]\npub fn top_level_cfg_test_helper() -> u32 {\n    99u32\n}\n\n",
        );
        interleaved.push_str(
            "#[cfg(test)]\nimpl AlphaSensor {\n    pub fn test_backdoor(&self) -> u32 {\n        self.reading\n    }\n}\n\n",
        );
        interleaved.push_str("/// Trailing public reader.\n/// Realises: [REQ-0608]\n");
        interleaved.push_str(
            "pub fn read_after(sample: u32) -> u32 {\n    sample.saturating_add(1)\n}\n\n",
        );
        interleaved
            .push_str("/// Non-test cfg(not(test)) production guard.\n/// Realises: [REQ-0608]\n");
        interleaved
            .push_str("#[cfg(not(test))]\npub fn prod_only_guard() -> u32 {\n    7u32\n}\n\n");
        interleaved.push_str("/// Crate-internal helper.\n/// Realises: [REQ-0608]\n");
        interleaved.push_str("pub(crate) fn crate_scoped_helper(v: u32) -> u32 {\n    v\n}\n\n");
        interleaved.push_str("/// Super-scoped internal record.\n/// Realises: [REQ-0608]\n");
        interleaved.push_str("pub(super) struct SuperScopedRecord;\n\n");
        interleaved.push_str("#[doc = \"Private kernel summary.\"]\n/// Realises: [REQ-0608]\n");
        interleaved.push_str("fn private_kernel(v: u32) -> u32 {\n    // non-exec comment\n    v.saturating_add(2)\n}\n");

        let cst = crate::parse_verified_lossless_cst(
            "src/interleaved.rs",
            interleaved.as_bytes(),
            &thresholds,
        )?;
        let inv = crate::classify_cfg_test_excluded_doc_tier(&cst, &thresholds, false)?;
        assert_eq!(inv.source_path, "src/interleaved.rs");
        assert_eq!(inv.rust_tier_a_count, 4);
        assert_eq!(inv.rust_tier_bc_count, 3);
        assert_eq!(inv.test_excluded_item_count, 7);
        assert_eq!(inv.pruned_test_subtree_ranges.len(), 4);
        assert_eq!(inv.production_items.len(), 7);
        assert_eq!(inv.production_exec_sloc, inv.production_exec_lines.len());
        assert_eq!(inv.production_exec_sloc, 20);

        let ordered_tiers: Vec<(&str, crate::DocTier, usize, bool)> = inv
            .production_items
            .values()
            .map(|item| {
                (
                    item.symbol.as_str(),
                    item.tier,
                    item.doc_line_count,
                    item.is_callable,
                )
            })
            .collect();
        assert_eq!(
            ordered_tiers,
            vec![
                ("AlphaSensor", crate::DocTier::RustTierA, 2, false),
                ("read_before", crate::DocTier::RustTierA, 2, true),
                ("read_after", crate::DocTier::RustTierA, 2, true),
                ("prod_only_guard", crate::DocTier::RustTierA, 2, true),
                ("crate_scoped_helper", crate::DocTier::RustTierBC, 2, true),
                ("SuperScopedRecord", crate::DocTier::RustTierBC, 2, false),
                ("private_kernel", crate::DocTier::RustTierBC, 2, true),
            ]
        );

        // 3. Generated self-hosted promotion (explicit flag & .g.rs path) via auto-cleaned temp dir.
        let self_hosted_src = concat!(
            "/// Public generated entrypoint.\n/// Realises: [REQ-0608]\n",
            "pub fn gen_api() -> u32 { 1 }\n",
            "/// Crate-internal generated helper.\n/// Realises: [REQ-0608]\n",
            "pub(crate) fn gen_internal() -> u32 { 2 }\n",
            "/// Private generated helper.\n/// Realises: [REQ-0608]\n",
            "fn gen_private() -> u32 { 3 }\n",
            "#[cfg(test)]\nmod gen_tests {\n    #[test]\n    fn gen_unit() { assert_eq!(1, 1); }\n}\n",
        );
        let temp_dir =
            std::env::temp_dir().join(format!(".deap_td06_doc_tier_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir)?;
        let temp_file = temp_dir.join("synthesized_contracts.g.rs");
        std::fs::write(&temp_file, self_hosted_src.as_bytes())?;
        let disk_bytes = std::fs::read(&temp_file);
        let _ = std::fs::remove_dir_all(&temp_dir);
        let disk_bytes = disk_bytes?;

        let manual_path_cst =
            crate::parse_verified_lossless_cst("src/manual_gen.rs", &disk_bytes, &thresholds)?;
        let promoted_by_flag =
            crate::classify_cfg_test_excluded_doc_tier(&manual_path_cst, &thresholds, true)?;
        assert_eq!(promoted_by_flag.rust_tier_a_count, 3);
        assert_eq!(promoted_by_flag.rust_tier_bc_count, 0);
        assert_eq!(promoted_by_flag.test_excluded_item_count, 2);

        let g_rs_cst = crate::parse_verified_lossless_cst(
            "src/synthesized_contracts.g.rs",
            &disk_bytes,
            &thresholds,
        )?;
        let promoted_by_path =
            crate::classify_cfg_test_excluded_doc_tier(&g_rs_cst, &thresholds, false)?;
        assert_eq!(promoted_by_path.rust_tier_a_count, 3);
        assert_eq!(promoted_by_path.rust_tier_bc_count, 0);
        assert_eq!(promoted_by_path.test_excluded_item_count, 2);

        // 4. Live deap-core module sanity (error.rs & lib.rs).
        let error_cst = crate::parse_verified_lossless_cst(
            "crates/deap-core/src/error.rs",
            include_str!("error.rs").as_bytes(),
            &thresholds,
        )?;
        let error_inv = crate::classify_cfg_test_excluded_doc_tier(&error_cst, &thresholds, false)?;
        assert_eq!(error_inv.rust_tier_a_count, 8);
        assert_eq!(error_inv.rust_tier_bc_count, 8);
        assert_eq!(error_inv.test_excluded_item_count, 3);

        let lib_cst = crate::parse_verified_lossless_cst(
            "crates/deap-core/src/lib.rs",
            include_str!("lib.rs").as_bytes(),
            &thresholds,
        )?;
        let lib_inv = crate::classify_cfg_test_excluded_doc_tier(&lib_cst, &thresholds, false)?;
        assert!(lib_inv.production_items.is_empty());
        assert_eq!(lib_inv.test_excluded_item_count, 2);

        // 5. Fail-closed fault-injection suite (DEAP-E0801..DEAP-E0804).
        let mut unsealed_cst = cst.clone();
        unsealed_cst.lossless_red_green_tree_ready = false;
        let precond_err =
            match crate::classify_cfg_test_excluded_doc_tier(&unsealed_cst, &thresholds, false) {
                Ok(_) => return Err("expected unsealed CST to fail closed with DEAP-E0801".into()),
                Err(err) => err,
            };
        assert_eq!(precond_err.diagnostic_code(), "DEAP-E0801");
        assert_eq!(
            precond_err,
            CstMetricsError::UnverifiedParsePrecondition {
                state: "AnalyzingMetricsAndTiers",
            }
        );

        let mut diverged_cst = crate::parse_verified_lossless_cst(
            "src/ascii_doc.rs",
            b"/// abcd\npub fn target() {}\n",
            &thresholds,
        )?;
        diverged_cst.source_text = "/// \u{03B1}\u{03B2}\npub fn target() {}\n".to_string();
        let span_err =
            match crate::classify_cfg_test_excluded_doc_tier(&diverged_cst, &thresholds, false) {
                Ok(_) => {
                    return Err("expected diverged GreenNode/UTF-8 span trap DEAP-E0802".into())
                }
                Err(err) => err,
            };
        assert_eq!(span_err.diagnostic_code(), "DEAP-E0802");

        let drifted_thresholds = CstMetricsThresholds {
            max_fn_param_count: 6,
            ..thresholds
        };
        let thr_err =
            match crate::classify_cfg_test_excluded_doc_tier(&cst, &drifted_thresholds, false) {
                Ok(_) => return Err("expected drifted thresholds to fail closed DEAP-E0803".into()),
                Err(err) => err,
            };
        assert_eq!(thr_err.diagnostic_code(), "DEAP-E0803");

        let unclosed_cfg = b"#[cfg(test\npub fn leaked_prod() {}\n";
        let unclosed_err = match crate::parse_verified_lossless_cst(
            "src/unclosed_cfg.rs",
            unclosed_cfg,
            &thresholds,
        ) {
            Ok(_) => return Err("expected unclosed #[cfg(test to fail closed DEAP-E0803".into()),
            Err(err) => err,
        };
        assert_eq!(unclosed_err.diagnostic_code(), "DEAP-E0803");

        // Calibrate depth-65 Red/Green CST fault injection into classify_cfg_test_excluded_doc_tier.
        let seed_layers = 8usize;
        let seed_src = build_nested_paren_rust_fixture(seed_layers);
        let seed_cst =
            crate::parse_verified_lossless_cst("src/seed.rs", seed_src.as_bytes(), &thresholds)?;
        let depth_65_layers = seed_layers
            .saturating_add(MAX_AST_DEPTH.saturating_sub(seed_cst.observed_max_depth()))
            .saturating_add(1);
        let depth_65_src = build_nested_paren_rust_fixture(depth_65_layers);
        let raw_parse_65 =
            ra_ap_syntax::SourceFile::parse(&depth_65_src, ra_ap_syntax::Edition::CURRENT);
        let injected_65 = VerifiedLosslessCst {
            source_path: "src/injected_depth_65.rs".to_string(),
            source_text: depth_65_src.clone(),
            green_node: raw_parse_65.syntax_node().green().to_owned(),
            source_file: raw_parse_65.tree(),
            line_starts: compute_monotone_line_starts(&depth_65_src)?,
            observed_max_depth: 65,
            verified_node_count: 1,
            lossless_red_green_tree_ready: true,
        };
        let depth_err =
            match crate::classify_cfg_test_excluded_doc_tier(&injected_65, &thresholds, false) {
                Ok(_) => return Err("expected depth 65 CST to fail closed with DEAP-E0804".into()),
                Err(err) => err,
            };
        assert_eq!(depth_err.diagnostic_code(), "DEAP-E0804");
        assert!(matches!(
            depth_err,
            CstMetricsError::AstDepthOrArithmeticExhaustion {
                observed_depth: 65,
                max_depth: MAX_AST_DEPTH,
                ..
            }
        ));

        Ok(())
    }

    /// Synthesizes an inherent method body with an exact target executable SLOC count (`>= 5`).
    /// Realises: [REQ-0608]
    fn build_exact_exec_sloc_method_fixture(fn_name: &str, target_exec_sloc: usize) -> String {
        let mut out = String::from("impl BoundaryProbe {\n");
        out.push_str("    /// Boundary method docstring excluded from exec_sloc.\n");
        out.push_str("    /// Realises: [REQ-0608]\n");
        out.push_str(&format!(
            "    pub fn {fn_name}(&mut self, p1: u32, p2: u32, p3: u32, p4: u32, p5: u32) -> Result<u32, CstMetricsError> {{\n"
        ));
        out.push_str("        // Comment and blank line below are non-executable trivia.\n\n");
        out.push_str("        let mut acc: u32 = p1.saturating_add(p2);\n");
        out.push_str("        if p1 > 0 && p2 > 0 || p3 > 0 {\n");
        let filler_lines = target_exec_sloc.saturating_sub(6);
        for idx in 0..filler_lines {
            out.push_str(&format!(
                "            acc = acc.saturating_add({idx}u32).saturating_add(p4).saturating_add(p5);\n"
            ));
        }
        out.push_str("        }\n");
        out.push_str("        Ok(acc)\n");
        out.push_str("    }\n");
        out.push_str("}\n");
        out
    }

    /// Verifies `FnAstMetrics`, `DeapV2AstMetricsReport`, `extract_callable_v2_metrics`, and `analyze_cst_v2_metrics` (`MT-US56-03`).
    /// Realises: [REQ-0608]
    #[test]
    fn extracts_exact_v2_metrics_sloc_arity_mut_bool_and_string_err(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let thresholds = crate::CstMetricsThresholds::default();

        // 1. Nominal lossless CST round-trip, receiver exclusion (self/&self/&mut self), #[rt_loop], and typed Result (AC-US-56-01).
        let nominal_src = concat!(
            "//! Nominal v2_metrics fixture with UTF-8 \u{03B1}\u{03B2}.\n\n",
            "/// Sensor frame container.\n/// Realises: [REQ-0608]\n",
            "pub struct SensorFrame {\n    pub ticks: u32,\n}\n\n",
            "impl SensorFrame {\n",
            "    /// Consumes value self receiver.\n    /// Realises: [REQ-0608]\n",
            "    pub fn into_ticks(self, scale: u32) -> u32 {\n",
            "        self.ticks.saturating_mul(scale)\n",
            "    }\n\n",
            "    /// Borrows shared &self receiver.\n    /// Realises: [REQ-0608]\n",
            "    pub fn check_bounds(&self, lo: u32, hi: u32) -> bool {\n",
            "        self.ticks >= lo && self.ticks <= hi\n",
            "    }\n\n",
            "    /// Borrows exclusive &mut self receiver and carries #[rt_loop].\n    /// Realises: [REQ-0608]\n",
            "    #[rt_loop]\n",
            "    pub fn step_rt(&mut self, delta: u32, ceiling: u32) -> Result<Vec<String>, CstMetricsError> {\n",
            "        let raw_brace = r#\"{ unbalanced raw string }\"#;\n",
            "        if delta > 0 && self.ticks < ceiling || raw_brace.is_empty() {\n",
            "            self.ticks = self.ticks.saturating_add(delta);\n",
            "        }\n",
            "        Ok(Vec::new())\n",
            "    }\n",
            "}\n\n",
            "/// Internal helper returning single-arg io::Result<String> (not Result<T, String>).\n/// Realises: [REQ-0608]\n",
            "#[deap::rt_loop]\n",
            "fn internal_rt_io(label: &str) -> std::io::Result<String> {\n",
            "    Ok(label.to_string())\n",
            "}\n",
        );

        let temp_dir =
            std::env::temp_dir().join(format!(".deap_td07_v2_metrics_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir)?;
        let temp_rs = temp_dir.join("nominal_metrics.rs");
        std::fs::write(&temp_rs, nominal_src.as_bytes())?;
        let disk_bytes = std::fs::read(&temp_rs);
        let _ = std::fs::remove_dir_all(&temp_dir);
        let disk_bytes = disk_bytes?;

        let nominal_cst =
            crate::parse_verified_lossless_cst("src/nominal_metrics.rs", &disk_bytes, &thresholds)?;
        let report: crate::DeapV2AstMetricsReport =
            crate::analyze_cst_v2_metrics(&nominal_cst, &thresholds)?;
        assert_eq!(report.source_path, "src/nominal_metrics.rs");
        assert!(report.lossless_roundtrip_verified);
        assert!(report.observed_max_depth <= MAX_AST_DEPTH);
        assert_eq!(report.rust_tier_a_count, 4);
        assert_eq!(report.rust_tier_bc_count, 1);
        assert_eq!(report.test_excluded_item_count, 0);
        assert_eq!(
            report.production_exec_sloc,
            report.production_exec_lines.len()
        );
        assert_eq!(report.callable_metrics.len(), 4);

        let ordered_names: Vec<&str> = report
            .callable_metrics
            .values()
            .map(|m| m.symbol.as_str())
            .collect();
        assert_eq!(
            ordered_names,
            vec!["into_ticks", "check_bounds", "step_rt", "internal_rt_io"]
        );

        let by_name: BTreeMap<&str, &crate::FnAstMetrics> = report
            .callable_metrics
            .values()
            .map(|m| (m.symbol.as_str(), m))
            .collect();

        let into_ticks = by_name.get("into_ticks").ok_or("missing into_ticks")?;
        assert!(into_ticks.has_self_receiver);
        assert_eq!(into_ticks.param_count, 1);
        assert_eq!(into_ticks.exec_sloc, 3);
        assert_eq!(into_ticks.exec_sloc, into_ticks.exec_lines.len());
        assert_eq!(into_ticks.mut_bool_flag_count, 0);
        assert_eq!(into_ticks.max_predicate_clauses, 0);
        assert!(!into_ticks.is_rt_loop);
        assert!(!into_ticks.returns_result_string_err);

        let check_bounds = by_name.get("check_bounds").ok_or("missing check_bounds")?;
        assert!(check_bounds.has_self_receiver);
        assert_eq!(check_bounds.param_count, 2);
        assert_eq!(check_bounds.exec_sloc, 3);
        assert_eq!(check_bounds.max_predicate_clauses, 2);
        assert!(!check_bounds.is_rt_loop);
        assert!(!check_bounds.returns_result_string_err);

        let step_rt = by_name.get("step_rt").ok_or("missing step_rt")?;
        assert!(step_rt.has_self_receiver);
        assert_eq!(step_rt.param_count, 2);
        assert_eq!(step_rt.exec_sloc, 8);
        assert_eq!(step_rt.mut_bool_flag_count, 0);
        assert_eq!(step_rt.max_predicate_clauses, 3);
        assert!(step_rt.is_rt_loop);
        assert!(!step_rt.returns_result_string_err);

        let internal_rt_io = by_name
            .get("internal_rt_io")
            .ok_or("missing internal_rt_io")?;
        assert!(!internal_rt_io.has_self_receiver);
        assert_eq!(internal_rt_io.param_count, 1);
        assert_eq!(internal_rt_io.exec_sloc, 4);
        assert!(internal_rt_io.is_rt_loop);
        assert!(!internal_rt_io.returns_result_string_err);

        // Direct extract_callable_v2_metrics parity check on every ast::Fn in nominal_cst.
        for node in nominal_cst.syntax_node().descendants() {
            if let Some(fn_ast) = ra_ap_syntax::ast::Fn::cast(node) {
                let direct: crate::FnAstMetrics =
                    crate::extract_callable_v2_metrics(&fn_ast, nominal_cst.line_starts())?;
                let key = (direct.byte_start as u32, direct.symbol.clone());
                assert_eq!(report.callable_metrics.get(&key), Some(&direct));
            }
        }

        // 2. Violation detection: mut bool flags, 4-clause predicate, Result<T, String> (AC-US-56-02).
        let violation_src = concat!(
            "pub fn bad_violations(a: bool, b: bool, c: bool, d: bool) -> Result<usize, String> {\n",
            "    let mut flag_explicit: bool = false;\n",
            "    let mut flag_inferred = true;\n",
            "    let (mut flag_tuple, count): (bool, usize) = (false, 1);\n",
            "    if a && b && c && d {\n",
            "        flag_explicit = true;\n",
            "    }\n",
            "    if flag_explicit || flag_inferred || flag_tuple {\n",
            "        return Ok(count);\n",
            "    }\n",
            "    Err(String::new())\n",
            "}\n",
            "fn qualified_string_err() -> std::result::Result<(), std::string::String> {\n",
            "    Ok(())\n",
            "}\n",
        );
        let violation_cst = crate::parse_verified_lossless_cst(
            "src/violations.rs",
            violation_src.as_bytes(),
            &thresholds,
        )?;
        let violation_report = crate::analyze_cst_v2_metrics(&violation_cst, &thresholds)?;
        let bad_fn = violation_report
            .callable_metrics
            .values()
            .find(|m| m.symbol == "bad_violations")
            .ok_or("missing bad_violations")?;
        assert_eq!(bad_fn.param_count, 4);
        assert_eq!(bad_fn.mut_bool_flag_count, 3);
        assert_eq!(bad_fn.max_predicate_clauses, 4);
        assert!(bad_fn.returns_result_string_err);

        let qual_fn = violation_report
            .callable_metrics
            .values()
            .find(|m| m.symbol == "qualified_string_err")
            .ok_or("missing qualified_string_err")?;
        assert!(qual_fn.returns_result_string_err);

        // 3. Exact C_CST_01 (50 SLOC vs 51 SLOC), C_CST_02 (5 params + &mut self vs 6 params), & MAX_AST_DEPTH = 64 boundary (AC-US-56-03).
        let mut boundary_src = String::from("pub struct BoundaryProbe;\n");
        boundary_src.push_str(&build_exact_exec_sloc_method_fixture("boundary_50", 50));
        boundary_src.push_str(&build_exact_exec_sloc_method_fixture("breach_51", 51));
        boundary_src.push_str(
            "pub fn six_params(a: u8, b: u8, c: u8, d: u8, e: u8, f: u8) -> u8 { a.saturating_add(b).saturating_add(c).saturating_add(d).saturating_add(e).saturating_add(f) }\n",
        );
        let boundary_cst = crate::parse_verified_lossless_cst(
            "src/boundary.rs",
            boundary_src.as_bytes(),
            &thresholds,
        )?;
        let boundary_report = crate::analyze_cst_v2_metrics(&boundary_cst, &thresholds)?;
        let b50 = boundary_report
            .callable_metrics
            .values()
            .find(|m| m.symbol == "boundary_50")
            .ok_or("missing boundary_50")?;
        assert!(b50.has_self_receiver);
        assert_eq!(b50.param_count, 5);
        assert_eq!(b50.exec_sloc, 50);
        assert_eq!(b50.mut_bool_flag_count, 0);
        assert_eq!(b50.max_predicate_clauses, 3);
        assert!(!b50.returns_result_string_err);

        let b51 = boundary_report
            .callable_metrics
            .values()
            .find(|m| m.symbol == "breach_51")
            .ok_or("missing breach_51")?;
        assert_eq!(b51.exec_sloc, 51);

        let p6 = boundary_report
            .callable_metrics
            .values()
            .find(|m| m.symbol == "six_params")
            .ok_or("missing six_params")?;
        assert!(!p6.has_self_receiver);
        assert_eq!(p6.param_count, 6);

        let seed_layers = 8usize;
        let seed_src = build_nested_paren_rust_fixture(seed_layers);
        let seed_cst =
            crate::parse_verified_lossless_cst("src/seed.rs", seed_src.as_bytes(), &thresholds)?;
        let exact_64_layers =
            seed_layers.saturating_add(MAX_AST_DEPTH.saturating_sub(seed_cst.observed_max_depth()));
        let depth_64_src = build_nested_paren_rust_fixture(exact_64_layers);
        let depth_64_cst = crate::parse_verified_lossless_cst(
            "src/depth_64.rs",
            depth_64_src.as_bytes(),
            &thresholds,
        )?;
        let depth_64_report = crate::analyze_cst_v2_metrics(&depth_64_cst, &thresholds)?;
        assert_eq!(depth_64_report.observed_max_depth, MAX_AST_DEPTH);
        assert_eq!(depth_64_report.callable_metrics.len(), 1);

        // 4. Subtree-scoped #[cfg(test)] & #[test] pruning without scope leakage (AC-US-56-04).
        let mut cfg_test_src = String::new();
        cfg_test_src.push_str(
            "/// Preceding production fn.\n/// Realises: [REQ-0608]\npub fn prod_head(v: u32) -> u32 {\n    v\n}\n\n",
        );
        cfg_test_src.push_str("#[cfg(test)]\nmod tests {\n");
        cfg_test_src.push_str(&build_seventy_sloc_test_helper_fn());
        cfg_test_src.push_str(
            "    #[test]\n    fn bad_in_test() -> Result<(), String> {\n        let mut f: bool = false;\n        let _ = f;\n        Ok(())\n    }\n",
        );
        cfg_test_src.push_str("}\n\n");
        cfg_test_src.push_str(
            "#[test]\nfn standalone_test() -> Result<(), String> {\n    let mut t = true;\n    let _ = t;\n    Ok(())\n}\n\n",
        );
        cfg_test_src.push_str(
            "/// Trailing production fn.\n/// Realises: [REQ-0608]\npub fn prod_tail(v: u32) -> u32 {\n    v.saturating_add(1)\n}\n",
        );
        let cfg_cst = crate::parse_verified_lossless_cst(
            "src/cfg_pruned.rs",
            cfg_test_src.as_bytes(),
            &thresholds,
        )?;
        let cfg_report = crate::analyze_cst_v2_metrics(&cfg_cst, &thresholds)?;
        let prod_fn_names: Vec<&str> = cfg_report
            .callable_metrics
            .values()
            .map(|m| m.symbol.as_str())
            .collect();
        assert_eq!(prod_fn_names, vec!["prod_head", "prod_tail"]);
        assert_eq!(cfg_report.production_exec_sloc, 6);
        assert!(cfg_report.test_excluded_item_count >= 3);

        // 5. Self-audit TD-07 production implementation complexity budget (<= 48 exec SLOC, 2 params, 0 mut bool, zero String err).
        let self_src = include_str!("source_ast.rs");
        let td07_marker = "/// Per-callable `v2_metrics` structural complexity record";
        let test_marker = ["\n#[cfg(test)]", "\nmod tests"].concat();
        let (_, from_td07) = self_src
            .split_once(td07_marker)
            .ok_or("missing FnAstMetrics in source_ast.rs")?;
        let (td07_body, _) = from_td07
            .split_once(&test_marker)
            .ok_or("missing #[cfg(test)] mod tests in source_ast.rs")?;
        let td07_slice = format!("{td07_marker}{td07_body}\n");
        let td07_cst = crate::parse_verified_lossless_cst(
            "crates/deap-core/src/source_ast.rs",
            td07_slice.as_bytes(),
            &thresholds,
        )?;
        let td07_report = crate::analyze_cst_v2_metrics(&td07_cst, &thresholds)?;
        for target_fn in ["extract_callable_v2_metrics", "analyze_cst_v2_metrics"] {
            let m = td07_report
                .callable_metrics
                .values()
                .find(|item| item.symbol == target_fn)
                .ok_or_else(|| format!("missing {target_fn} in TD-07 report"))?;
            assert_eq!(m.param_count, 2, "{target_fn} must take 2 params");
            assert!(
                m.exec_sloc <= 48,
                "{target_fn} exec_sloc {} exceeded 48",
                m.exec_sloc
            );
            assert_eq!(m.mut_bool_flag_count, 0);
            assert!(m.max_predicate_clauses <= 3);
            assert!(!m.returns_result_string_err);
        }
        for helper_m in td07_report.callable_metrics.values() {
            assert!(helper_m.exec_sloc <= 50);
            assert!(helper_m.param_count <= 5);
            assert_eq!(helper_m.mut_bool_flag_count, 0);
            assert!(helper_m.max_predicate_clauses <= 3);
            assert!(!helper_m.returns_result_string_err);
        }
        assert_eq!(td07_report.rust_tier_a_count, 4);
        for item in td07_report.doc_tier_inventory.production_items.values() {
            match item.tier {
                crate::DocTier::RustTierA => assert!(item.doc_line_count >= 7),
                crate::DocTier::RustTierBC => assert_eq!(
                    item.doc_line_count, 2,
                    "RustTierBC item '{}' must have exact 2-line doc header",
                    item.symbol
                ),
                crate::DocTier::TestExcluded => {
                    return Err("unexpected TestExcluded item in production_items".into());
                }
            }
        }

        let live_err_cst = crate::parse_verified_lossless_cst(
            "crates/deap-core/src/error.rs",
            include_str!("error.rs").as_bytes(),
            &thresholds,
        )?;
        let live_err_report = crate::analyze_cst_v2_metrics(&live_err_cst, &thresholds)?;
        assert!(live_err_report
            .callable_metrics
            .values()
            .all(|m| !m.returns_result_string_err));

        // 6. Fail-closed fault injection: DEAP-E0801..DEAP-E0804 (AC-US-56-05..AC-US-56-08).
        let mut unsealed = nominal_cst.clone();
        unsealed.lossless_red_green_tree_ready = false;
        let e0801 = match crate::analyze_cst_v2_metrics(&unsealed, &thresholds) {
            Ok(_) => return Err("expected unsealed CST to fail closed with DEAP-E0801".into()),
            Err(err) => err,
        };
        assert_eq!(e0801.diagnostic_code(), "DEAP-E0801");

        let mut diverged = crate::parse_verified_lossless_cst(
            "src/diverged.rs",
            b"/// abcd\npub fn f() {}\n",
            &thresholds,
        )?;
        diverged.source_text = "/// \u{03B1}\u{03B2}\npub fn f() {}\n".to_string();
        let e0802 = match crate::analyze_cst_v2_metrics(&diverged, &thresholds) {
            Ok(_) => return Err("expected diverged CST to fail closed with DEAP-E0802".into()),
            Err(err) => err,
        };
        assert_eq!(e0802.diagnostic_code(), "DEAP-E0802");

        let sample_fn = nominal_cst
            .syntax_node()
            .descendants()
            .find_map(ra_ap_syntax::ast::Fn::cast)
            .ok_or("missing sample ast::Fn")?;
        let bad_lines_err = match crate::extract_callable_v2_metrics(&sample_fn, &[]) {
            Ok(_) => return Err("expected empty line_starts to fail closed with DEAP-E0802".into()),
            Err(err) => err,
        };
        assert_eq!(bad_lines_err.diagnostic_code(), "DEAP-E0802");

        let bad_thr = CstMetricsThresholds {
            max_fn_exec_sloc: 51,
            ..thresholds
        };
        let e0803 = match crate::analyze_cst_v2_metrics(&nominal_cst, &bad_thr) {
            Ok(_) => {
                return Err("expected drifted thresholds to fail closed with DEAP-E0803".into())
            }
            Err(err) => err,
        };
        assert_eq!(e0803.diagnostic_code(), "DEAP-E0803");

        let depth_65_src = build_nested_paren_rust_fixture(exact_64_layers.saturating_add(1));
        let raw_65 = ra_ap_syntax::SourceFile::parse(&depth_65_src, ra_ap_syntax::Edition::CURRENT);
        let depth_65_lines = compute_monotone_line_starts(&depth_65_src)?;
        let depth_65_fn = raw_65
            .tree()
            .syntax()
            .descendants()
            .find_map(ra_ap_syntax::ast::Fn::cast)
            .ok_or("missing depth_65 ast::Fn")?;
        let fn_e0804 = match crate::extract_callable_v2_metrics(&depth_65_fn, &depth_65_lines) {
            Ok(_) => return Err("expected depth 65 fn to fail closed with DEAP-E0804".into()),
            Err(err) => err,
        };
        assert_eq!(fn_e0804.diagnostic_code(), "DEAP-E0804");

        let injected_65 = VerifiedLosslessCst {
            source_path: "src/depth_65.rs".to_string(),
            source_text: depth_65_src,
            green_node: raw_65.syntax_node().green().to_owned(),
            source_file: raw_65.tree(),
            line_starts: depth_65_lines,
            observed_max_depth: 65,
            verified_node_count: 1,
            lossless_red_green_tree_ready: true,
        };
        let cst_e0804 = match crate::analyze_cst_v2_metrics(&injected_65, &thresholds) {
            Ok(_) => return Err("expected depth 65 CST to fail closed with DEAP-E0804".into()),
            Err(err) => err,
        };
        assert_eq!(cst_e0804.diagnostic_code(), "DEAP-E0804");

        Ok(())
    }
}

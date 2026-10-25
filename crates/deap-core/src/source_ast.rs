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
//! Realises: [REQ-FEAT-42-SOURCE-AST], [DO-330-TQL-4-SOURCE-AST]

use crate::sysml_ast::{
    ActionDef, AttributeDef, ConstraintDef, PackageDef, PartDef, PortDef, RequirementDef, StateDef,
};
use std::collections::BTreeSet;
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SourceLanguage {
    /// Automatically detect `.rs` and `.dart` files from file extensions.
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
    pub fn from_flag(flag: &str) -> Result<Self, String> {
        let normalized = flag.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "auto" | "" => Ok(Self::Auto),
            "rust" | "rs" => Ok(Self::Rust),
            "dart" => Ok(Self::Dart),
            other => Err(format!(
                "Unsupported source language '{}'. Expected one of: auto, rust, dart.",
                other
            )),
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
pub fn validate_source_syntax(source: &str, file_label: &str) -> Result<(), String> {
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
type InterpolationFrame = (ActiveStringState, usize);

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
/// Converts a callable `SourceAstItem` (`Function` or `Method`) into `(in_parameters, out_parameters, return_type_name)`
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
/// # Realises
/// - Realises: [REQ-0601], [REQ-FEAT-42-EXTRACT-METHOD-SIGNATURE]
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
/// # Realises
/// - Realises: [REQ-0601], [REQ-FEAT-42-EXTRACT-COMPONENT-METHOD-TABLE]
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
        let Some(sig) = extract_method_signature(item) else {
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
) -> Result<(), String> {
    let bytes = source.as_bytes();
    let mut idx: usize = 0;
    let mut line: usize = 1;
    let mut col: usize = 1;
    let mut stack: Vec<(u8, usize, usize)> = Vec::new();
    let mut active_str: Option<ActiveStringState> = None;
    let mut interp_stack: Vec<InterpolationFrame> = Vec::new();
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
                    return Err(format!(
                        "{}:{}:{}: syntax error: unclosed single-quoted string at line {}, column {}",
                        file_label, open_line, open_col, open_line, open_col
                    ));
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

        // Block comment `/* ... */`
        if b == b'/' && bytes.get(idx.saturating_add(1)) == Some(&b'*') {
            let open_line = line;
            let open_col = col;
            idx = idx.saturating_add(2);
            col = col.saturating_add(2);
            let mut closed = false;
            while let Some(&cb) = bytes.get(idx) {
                if cb == b'*' && bytes.get(idx.saturating_add(1)) == Some(&b'/') {
                    idx = idx.saturating_add(2);
                    col = col.saturating_add(2);
                    closed = true;
                    break;
                }
                if cb == b'\n' {
                    line = line.saturating_add(1);
                    col = 1;
                } else {
                    col = col.saturating_add(1);
                }
                idx = idx.saturating_add(1);
            }
            if !closed {
                return Err(format!(
                    "{}:{}:{}: syntax error: unclosed block comment starting at line {}, column {}",
                    file_label, open_line, open_col, open_line, open_col
                ));
            }
            continue;
        }

        // Rust raw string literal `r#"..."#`
        if !is_dart && b == b'r' {
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
                    return Err(format!(
                        "{}:{}:{}: syntax error: unclosed raw string literal at line {}, column {}",
                        file_label, open_line, open_col, open_line, open_col
                    ));
                }
                continue;
            }
        }

        let is_raw_quote = idx > 0
            && bytes.get(idx.saturating_sub(1)) == Some(&b'r')
            && (idx < 2
                || bytes
                    .get(idx.saturating_sub(2))
                    .map(|&p| !p.is_ascii_alphanumeric() && p != b'_')
                    .unwrap_or(true));

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
                    return Err(format!(
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
                    ));
                }
                None => {
                    return Err(format!(
                        "{}:{}:{}: syntax error: unexpected closing '{}' at line {}, column {} with no matching opening delimiter",
                        file_label,
                        line,
                        col,
                        char::from(b),
                        line,
                        col
                    ));
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
        return Err(format!(
            "{}:{}:{}: syntax error: unclosed {} at line {}, column {}",
            file_label, open_line, open_col, kind_label, open_line, open_col
        ));
    }

    if let Some((open_b, open_line, open_col)) = stack.pop() {
        return Err(format!(
            "{}:{}:{}: syntax error: unclosed delimiter '{}' opened at line {}, column {}",
            file_label,
            open_line,
            open_col,
            char::from(open_b),
            open_line,
            open_col
        ));
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
pub fn parse_rust_source_ast(source: &str, file_label: &str) -> Result<Vec<SourceAstItem>, String> {
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
                if let Some(impl_target) = parse_rust_impl_header(code) {
                    active_impl_owner = Some(impl_target);
                    current_doc.clear();
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
                        if kind == SourceItemKind::Struct && code.contains('{') && !code.contains('}') {
                            active_struct_idx = Some(item_idx);
                        } else if kind == SourceItemKind::Enum && code.contains('{') && !code.contains('}') {
                            active_enum_idx = Some(item_idx);
                        } else if kind == SourceItemKind::Trait && code.contains('{') && !code.contains('}') {
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
                        let actual_kind = if kind == SourceItemKind::Function && active_impl_owner.is_some() {
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
            active_impl_owner = None;
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
pub fn parse_dart_source_ast(source: &str, file_label: &str) -> Result<Vec<SourceAstItem>, String> {
    validate_source_syntax_for_language(source, file_label, SourceLanguage::Dart)?;

    let mut items: Vec<SourceAstItem> = Vec::new();
    let mut current_doc: Vec<String> = Vec::new();
    let mut in_multiline_comment = false;
    let mut brace_depth: usize = 0;
    let mut paren_depth: usize = 0;
    let mut active_class_idx: Option<usize> = None;
    let mut active_class_name: Option<String> = None;
    let mut active_fn_idx: Option<usize> = None;

    for (idx, raw_line) in source.lines().enumerate() {
        let line_no = idx.saturating_add(1);
        let trimmed = raw_line.trim();

        if in_multiline_comment {
            if trimmed.contains("*/") {
                in_multiline_comment = false;
            }
            continue;
        }
        if trimmed.starts_with("/*") && !trimmed.contains("*/") {
            in_multiline_comment = true;
            continue;
        }
        if trimmed.starts_with("///") {
            current_doc.push(trimmed.to_string());
            continue;
        }

        let code = strip_line_comment_preserving_strings(trimmed).trim();
        let stripped_code = strip_leading_dart_annotations(code);
        if stripped_code.trim().is_empty() {
            continue;
        }
        let code = stripped_code.trim();

        if paren_depth == 0 {
            if brace_depth == 0 {
                if let Some((kind, name, parent_type)) = classify_dart_top_level_ast(code) {
                    let reqs = extract_realises_ids_from_doc_lines(&current_doc);
                    let (params, return_type) = if kind.is_callable() {
                        parse_dart_callable_signature(code, &name)
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
                        let (params, return_type) =
                            parse_dart_callable_signature(code, &member_name);
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
            || (brace_depth == 0 && (code.ends_with(';') || code.ends_with('}')))
        {
            active_class_idx = None;
            active_class_name = None;
            active_fn_idx = None;
        } else if brace_depth == 1 {
            active_fn_idx = None;
        }
    }

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
                if let Some(ref parent) = item.parent_type {
                    if let Some(widget_name) = extract_state_generic_target(parent) {
                        if let Some(owner_part) = pkg
                            .part_defs
                            .iter_mut()
                            .find(|p| p.name == widget_name)
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
                }

                if let Some(existing) = pkg.part_defs.iter_mut().find(|p| p.name == part_name) {
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
        let Some((in_sig, out_sig, _)) = extract_method_signature(item) else {
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
                owner_part.actions.push(action_def);
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
            name: format!("{}Controller", clean_pkg_name),
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
) -> Result<PackageDef, String> {
    if !source_dir.exists() {
        return Err(format!(
            "Source path '{}' does not exist",
            source_dir.display()
        ));
    }

    let files = discover_source_files(source_dir, language)?;
    if files.is_empty() {
        return Err(format!(
            "No eligible '{}' source files (.rs / .dart) found in '{}'",
            language.label(),
            source_dir.display()
        ));
    }

    let mut all_items: Vec<SourceAstItem> = Vec::new();
    for file_path in &files {
        let content = fs::read_to_string(file_path).map_err(|e| {
            format!(
                "Failed to read source file '{}': {}",
                file_path.display(),
                e
            )
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
) -> Result<Vec<PathBuf>, String> {
    if root.is_file() {
        return Ok(vec![root.to_path_buf()]);
    }

    let mut files = Vec::new();
    for entry_res in WalkDir::new(root)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|e| {
            if e.file_type().is_dir() {
                let dname = e.file_name().to_string_lossy();
                !matches!(
                    dname.as_ref(),
                    "target" | ".git" | "build" | "Pods" | "ephemeral" | ".dart_tool" | "generated"
                )
            } else {
                true
            }
        })
    {
        let entry = entry_res
            .map_err(|e| format!("Directory walk error under '{}': {}", root.display(), e))?;
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
/// Realises: [REQ-FEAT-42-PARSE-RUST-IMPL-HEADER]
fn parse_rust_impl_header(code: &str) -> Option<String> {
    let after_unsafe = code.strip_prefix("unsafe ").unwrap_or(code).trim_start();
    let rest = if let Some(r) = after_unsafe.strip_prefix("impl<") {
        let mut depth: usize = 1;
        let mut end_idx: Option<usize> = None;
        for (i, ch) in r.char_indices() {
            if ch == '<' {
                depth = depth.saturating_add(1);
            } else if ch == '>' {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    end_idx = Some(i.saturating_add(1));
                    break;
                }
            }
        }
        r.get(end_idx?..).unwrap_or("").trim_start()
    } else {
        after_unsafe.strip_prefix("impl ")?.trim_start()
    };

    if !code.contains('{') {
        return None;
    }

    let before_brace = rest.split_once('{').map(|(b, _)| b.trim()).unwrap_or(rest);
    let target_part = if let Some((_, after_for)) = before_brace.split_once(" for ") {
        after_for.trim()
    } else {
        before_brace
    };
    let target_clean = target_part.trim_start_matches('&').trim_start_matches("mut ").trim();
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
/// - Strips visibility (`pub`, `pub(crate)`, `pub(super)`, `pub(in ...)`) and qualifiers (`const`, `async`, `unsafe`).
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
    if let Some(rest) = after_vis.strip_prefix("trait ") {
        let name = extract_leading_ast_ident(rest.trim_start())?;
        return Some((SourceItemKind::Trait, name));
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
            let pname = extract_leading_ast_ident(clean_pname)
                .unwrap_or_else(|| "param".to_string());
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
pub fn classify_dart_top_level_ast(
    code: &str,
) -> Option<(SourceItemKind, String, Option<String>)> {
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
                return Some((
                    SourceItemKind::Constructor,
                    format!("{}.{}", cls, named),
                ));
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
    let before_eq = without_semi
        .split_once('=')
        .map(|(b, _)| b.trim())
        .unwrap_or(without_semi);
    if before_eq.contains('(') && !before_eq.contains("Function(") {
        return None;
    }

    let mut rest = before_eq;
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

    // Extract trailing identifier as field name, and preceding part as type_name
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
        "String".to_string()
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

    let return_type = {
        let trimmed_before = before_open.trim();
        if let Some(prefix) = trimmed_before.strip_suffix(callable_name) {
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
        } else {
            None
        }
    };

    let param_section = after_open
        .rsplit_once(')')
        .map(|(inside, _)| inside)
        .unwrap_or(after_open);
    let cleaned_section = param_section
        .trim()
        .trim_start_matches('{')
        .trim_end_matches('}')
        .trim_start_matches('[')
        .trim_end_matches(']');

    for raw_param in split_top_level_comma(cleaned_section) {
        let mut p = raw_param
            .trim()
            .trim_start_matches('{')
            .trim_end_matches('}')
            .trim_start_matches('[')
            .trim_end_matches(']')
            .trim();
        if p.is_empty() {
            continue;
        }
        if let Some((before_default, _)) = p.split_once('=') {
            p = before_default.trim();
        }
        if let Some(after_req) = p.strip_prefix("required ") {
            p = after_req.trim();
        }
        if let Some(after_cov) = p.strip_prefix("covariant ") {
            p = after_cov.trim();
        }
        if let Some(after_final) = p.strip_prefix("final ") {
            p = after_final.trim();
        }

        if let Some(super_field) = p.strip_prefix("super.") {
            if let Some(ident) = extract_leading_ast_ident(super_field) {
                params.push(SourceParam {
                    name: ident,
                    type_name: "Key".to_string(),
                });
            }
            continue;
        }
        if let Some(this_field) = p.strip_prefix("this.") {
            if let Some(ident) = extract_leading_ast_ident(this_field) {
                params.push(SourceParam {
                    name: ident,
                    type_name: "String".to_string(),
                });
            }
            continue;
        }

        // Otherwise `<Type> <name>` or `<name>`
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
        let ptype_raw: String = rev_chars.collect::<String>().chars().rev().collect();
        if !pname.is_empty() {
            let ptype = if ptype_raw.trim().is_empty() {
                "String".to_string()
            } else {
                ptype_raw.trim().to_string()
            };
            params.push(SourceParam {
                name: pname,
                type_name: ptype,
            });
        }
    }

    (params, return_type)
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

    // If generic `Outer<Inner>`, extract `Outer` (or for Option/Future/Stream/Vec, preserve a built-in identifier)
    let base = if let Some((outer, _)) = s.split_once('<') {
        outer.trim()
    } else {
        s.trim()
    };

    let last_segment = base.rsplit_once("::").map(|(_, last)| last).unwrap_or(base);
    let ident = sanitize_sysml_identifier(last_segment, "String");
    match ident.as_str() {
        "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8" | "i16" | "i32" | "i64"
        | "i128" | "isize" | "int" => "Integer".to_string(),
        "f32" | "f64" | "double" | "num" => "Real".to_string(),
        "bool" => "Boolean".to_string(),
        "str" | "String" | "char" | "Path" | "PathBuf" => "String".to_string(),
        "Self" | "Result" | "Option" | "Vec" | "Box" | "Arc" | "Rc" | "Future" | "Stream" => {
            "String".to_string()
        }
        other if is_sysml_reserved_keyword(other) => "String".to_string(),
        other => other.to_string(),
    }
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
            trimmed_before.get(..idx).unwrap_or(trimmed_before).trim_end()
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
                if after_backslash.get(close_brace.saturating_add(1)..close_brace.saturating_add(2)) == Some("'") {
                    return Some(close_brace.saturating_add(4));
                }
            }
        } else if after_backslash.get(first.len_utf8()..first.len_utf8().saturating_add(1)) == Some("'") {
            return Some(first.len_utf8().saturating_add(3));
        }
        None
    } else {
        let mut chars = rest.chars();
        let ch = chars.next()?;
        if ch != '\'' && ch != '\\' && rest.get(ch.len_utf8()..ch.len_utf8().saturating_add(1)) == Some("'") {
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
                if bytes.get(i) == Some(&b'r') || bytes.get(i) == Some(&b'R') {
                    let prev_ok = i == 0
                        || bytes
                            .get(i.saturating_sub(1))
                            .map(|b| (!b.is_ascii_alphanumeric() && *b != b'_') || *b == b'b')
                            .unwrap_or(true);
                    if prev_ok {
                        let mut hashes: usize = 0;
                        while bytes.get(i.saturating_add(1).saturating_add(hashes)) == Some(&b'#')
                        {
                            hashes = hashes.saturating_add(1);
                        }
                        if bytes.get(i.saturating_add(1).saturating_add(hashes)) == Some(&b'"') {
                            *state = RustStringScannerState::RawString(hashes);
                            out.push('"');
                            i = i.saturating_add(2).saturating_add(hashes);
                            continue;
                        }
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

/// Detects a Rust hashed raw string prefix `r#"..."#` (or `R#"..."#`) at byte offset `idx`.
///
/// # Summary
/// Inspects `bytes` at `idx` for an `'r'` or `'R'` token (not preceded by an identifier character `[A-Za-z0-9_]`)
/// followed by `hashes >= 1` `'#'` bytes and an opening `'"'`, returning `Some((hashes, quote_idx))`.
///
/// # Preconditions
/// - `bytes` is a source line byte slice; `idx` is the candidate byte index of `'r'` or `'R'`.
///
/// # Postconditions
/// - Returns `Some((hashes, quote_idx))` where `hashes >= 1` and `bytes.get(quote_idx) == Some(&b'"')`, or `None`.
///
/// # Invariants
/// - Rejects identifier suffixes ending in `r` (such as `bar#"..."`) by verifying `idx == 0` or non-identifier predecessor.
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
/// and unbalanced delimiters inside `r#"..."#` literals.
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
    if b != b'r' && b != b'R' {
        return None;
    }
    if idx > 0 {
        let prev = *bytes.get(idx.saturating_sub(1))?;
        if prev.is_ascii_alphanumeric() || prev == b'_' {
            return None;
        }
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

    while let Some(&b) = bytes.get(i) {
        if let Some(iq) = inner_quote {
            if b == b'\\' {
                i = i.saturating_add(2);
                continue;
            }
            if b == iq {
                inner_quote = None;
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
        } else if b == b'}' {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return has_unescaped_closing_quote(bytes, i.saturating_add(1), quote);
            }
        } else if b == quote {
            if bytes.get(i.saturating_add(1)) == Some(&b';') {
                return false;
            }
            inner_quote = Some(b);
        } else if b == b'"' || b == b'\'' {
            inner_quote = Some(b);
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
    while prev > 0 && bytes.get(prev.saturating_sub(1)).map(|b| b.is_ascii_whitespace()).unwrap_or(false) {
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
/// Prevents URL protocols (`http://`, `package://`) and comments inside standard or hashed raw string literals
/// (`r#"..."#`) from being prematurely truncated as line comments in Rust and Dart source files.
///
/// ## 2. Invariants & Mathematical Properties
/// - Only truncates at `//` when neither `in_single_quote`, `in_double_quote`, nor `raw_hashes` is active.
/// - Accurately skips escaped quotes, hashed raw strings `r#"..."#`, and character literals.
///
/// ## 3. Preconditions
/// - `line` is a valid UTF-8 string slice.
///
/// ## 4. Postconditions
/// - Returns a prefix slice of `line` up to the first unquoted `//`, or the entire `line` if none exists.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(L) where L is byte length of `line`.
/// - Space Complexity: O(1) zero heap allocations.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Non-panicking byte scanner using `.get(..)`.
/// - Stateless pure function.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SOURCE-AST-STRIP-LINE-COMMENT]
pub fn strip_line_comment_preserving_strings(line: &str) -> &str {
    let bytes = line.as_bytes();
    let mut idx: usize = 0;
    let mut in_double_quote = false;
    let mut in_single_quote = false;
    let mut is_raw = false;
    let mut raw_hashes: Option<usize> = None;

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

        if !in_double_quote && !in_single_quote {
            if b == b'/' && bytes.get(idx.saturating_add(1)) == Some(&b'/') {
                return line.get(..idx).unwrap_or("");
            }

            if let Some((hashes, quote_idx)) = rust_raw_string_hash_count(bytes, idx) {
                raw_hashes = Some(hashes);
                idx = quote_idx.saturating_add(1);
                continue;
            }

            if (b == b'r' || b == b'R')
                && (bytes.get(idx.saturating_add(1)) == Some(&b'\'') || bytes.get(idx.saturating_add(1)) == Some(&b'"'))
            {
                let prev_is_ident = if idx > 0 {
                    bytes
                        .get(idx.saturating_sub(1))
                        .map(|&p| p.is_ascii_alphanumeric() || p == b'_')
                        .unwrap_or(false)
                } else {
                    false
                };
                if !prev_is_ident {
                    is_raw = true;
                    idx = idx.saturating_add(1);
                    continue;
                }
            }

            if b == b'"' {
                in_double_quote = true;
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
                    in_single_quote = true;
                }
                idx = idx.saturating_add(1);
                continue;
            }

            is_raw = false;
            idx = idx.saturating_add(1);
            continue;
        }

        if in_double_quote {
            if !is_raw && b == b'\\' {
                idx = idx.saturating_add(2);
                continue;
            }
            if b == b'"' {
                in_double_quote = false;
                is_raw = false;
            }
            idx = idx.saturating_add(1);
            continue;
        }

        if in_single_quote {
            if !is_raw && b == b'\\' {
                idx = idx.saturating_add(2);
                continue;
            }
            if b == b'\'' {
                in_single_quote = false;
                is_raw = false;
            }
            idx = idx.saturating_add(1);
            continue;
        }

        idx = idx.saturating_add(1);
    }

    line
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
pub fn strip_leading_dart_annotations(mut code: &str) -> &str {
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
        if rest.trim_start().starts_with('(') {
            let mut paren_depth: usize = 0;
            let mut paren_end = None;
            for (p_idx, b) in rest.char_indices() {
                if b == '(' {
                    paren_depth = paren_depth.saturating_add(1);
                } else if b == ')' {
                    paren_depth = paren_depth.saturating_sub(1);
                    if paren_depth == 0 {
                        paren_end = Some(p_idx.saturating_add(1));
                        break;
                    }
                }
            }
            if let Some(end) = paren_end {
                code = rest.get(end..).unwrap_or("");
                continue;
            }
        }
        code = rest;
    }
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

        if (b == b'r' || b == b'R')
            && (bytes.get(idx.saturating_add(1)) == Some(&b'\'') || bytes.get(idx.saturating_add(1)) == Some(&b'"'))
        {
            let prev_is_ident = if idx > 0 {
                bytes
                    .get(idx.saturating_sub(1))
                    .map(|&p| p.is_ascii_alphanumeric() || p == b'_')
                    .unwrap_or(false)
            } else {
                false
            };
            if !prev_is_ident {
                is_raw = true;
                idx = idx.saturating_add(1);
                continue;
            }
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
/// - Never panics (`!#[forbid(clippy::indexing_slicing)]` compliant via `.get(..)`).
///
/// # Safety
/// - Safe pure lexical byte scan (`!#[forbid(unsafe_code)]` compliant).
///
/// # Realises
/// - Realises: [REQ-SOURCE-AST-CHECK34-RANGE-SLICE-DETECTOR]
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and Check 34 (`check_zero_regex`) non-panicking slice invariants.
/// Provides general AST-level verification that production code never uses panicking `&s[a..b]` syntax.
///
/// ## 2. Invariants & Mathematical Properties
/// - Matches `[` at byte index `idx > 0` iff `bytes.get(idx - 1)` is in `[A-Za-z0-9_)\]]` and a `..`
///   token appears before the matching `]` at bracket depth 1.
///
/// ## 3. Preconditions
/// - `code_line: &str` is a comment-stripped source code line.
///
/// ## 4. Postconditions
/// - Returns `true` iff a direct range-slice indexing operation exists on `code_line`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) where N is `code_line.len()`.
/// - Space Complexity: O(1) zero heap allocation.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Non-panicking `.get(..)` byte access throughout.
/// - State: Stateless local indices.
/// - Action: Returns `false` on clean lines.
/// - Resource: Zero heap allocations.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SOURCE-AST-CHECK34-RANGE-SLICE-DETECTOR]
pub fn contains_direct_range_slice(code_line: &str) -> bool {
    let bytes = code_line.as_bytes();
    let mut idx: usize = 0;
    let mut in_double_quote = false;
    let mut raw_hashes: Option<usize> = None;

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

        if in_double_quote {
            if b == b'\\' {
                idx = idx.saturating_add(2);
                continue;
            }
            if b == b'"' {
                in_double_quote = false;
            }
            idx = idx.saturating_add(1);
            continue;
        }

        if b == b'"' {
            in_double_quote = true;
            idx = idx.saturating_add(1);
            continue;
        }

        if b == b'\'' {
            if let Some(char_len) = code_line.get(idx..).and_then(single_char_literal_byte_len) {
                idx = idx.saturating_add(char_len);
                continue;
            }
            idx = idx.saturating_add(1);
            continue;
        }

        if b == b'[' && idx > 0 {
            let prev_is_slice_target = bytes
                .get(idx.saturating_sub(1))
                .map(|&p| p.is_ascii_alphanumeric() || p == b'_' || p == b')' || p == b']')
                .unwrap_or(false);
            if prev_is_slice_target {
                let mut scan = idx.saturating_add(1);
                let mut depth: usize = 1;
                while let Some(&sb) = bytes.get(scan) {
                    if sb == b'[' {
                        depth = depth.saturating_add(1);
                    } else if sb == b']' {
                        depth = depth.saturating_sub(1);
                        if depth == 0 {
                            break;
                        }
                    } else if depth == 1
                        && sb == b'.'
                        && bytes.get(scan.saturating_add(1)) == Some(&b'.')
                    {
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
        assert!(panel.attributes.iter().any(|a| a.name == "title"));
        assert!(panel.attributes.iter().any(|a| a.name == "refreshRateHz"));
        assert!(panel.ports.iter().any(|p| p.name == "onRefresh"));
        assert!(panel.ports.iter().any(|p| p.name == "onRateChanged"));
        assert!(panel
            .satisfied_requirements
            .contains(&"REQ-UI-01".to_string()));
        assert!(!panel.states.is_empty());
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
        assert!(self.count <= self.capacity);
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
        assert_eq!(buf.attributes.len(), 2);
        assert!(buf.actions.iter().any(|a| a.name == "push_item"
            && a.in_params.len() == 1
            && a.out_params.len() == 1));
        assert!(!buf.constraints.is_empty());
        assert!(buf
            .satisfied_requirements
            .contains(&"REQ-BUF-01".to_string()));
        Ok(())
    }

    #[test]
    fn test_syntax_error_unclosed_brace_fails_closed_with_line_and_col() -> Result<(), String> {
        let bad_rust = "pub struct Broken {\n    pub x: u32,\n";
        let err = match parse_rust_source_ast(bad_rust, "src/broken.rs") {
            Ok(_) => return Err("Expected syntax error for unclosed brace".to_string()),
            Err(e) => e,
        };
        assert!(
            err.contains("src/broken.rs:1:19") && err.contains("unclosed delimiter '{'"),
            "Expected line/col diagnostic, got: {}",
            err
        );
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
    /// - Asserts exact stripped slices, delimiter depth counters, and absence of raw range slices or `#[allow(clippy::type_complexity)]` in production code.
    ///
    /// # Invariants
    /// - `strip_line_comment_preserving_strings` preserves `'/'`, `"http://..."`, and `r#"http://... // keep"#` while stripping trailing `//`.
    /// - `update_ast_delimiter_depths` ignores `'{'` and `'('` inside character literals, hashed raw strings, and unclosed `"${"` literals.
    /// - Production code prior to `#[cfg(test)]` contains zero raw range-slice indexing expressions and zero `#[allow(clippy::type_complexity)]` attributes.
    ///
    /// # Failure Modes
    /// - Panics or returns `Err` if comment stripping, delimiter depth tracking, or Check 34 self-verification assertions fail.
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
    /// - `Result<(), String>` (`Ok(())` on assertion success).
    ///
    /// # Errors
    /// - Returns `Err(String)` if any invariant check fails.
    ///
    /// # Panics
    /// - Panics if comment stripping, delimiter depth tracking, or range-slice purity assertions fail.
    ///
    /// # Safety
    /// - Safe pure lexical unit test (`!#[forbid(unsafe_code)]` compliant).
    ///
    /// # Realises
    /// - Realises: [REQ-SOURCE-AST-STRIP-LINE-COMMENT], [REQ-FEAT-42-UPDATE-AST-DELIMITER-DEPTHS]
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
    /// - Asserts exact stripped slices, delimiter depth counters, and absence of raw range slices in production code.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N) where N is byte length of `source_ast.rs`.
    /// - Space Complexity: O(1) auxiliary heap allocations.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Fails test execution immediately if any raw range-slice token or `#[allow(clippy::type_complexity)]` remains in production code.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SOURCE-AST-STRIP-LINE-COMMENT], [REQ-FEAT-42-UPDATE-AST-DELIMITER-DEPTHS]
    #[test]
    fn test_strip_comment_and_delimiter_depths_zero_range_slice_indexing() -> Result<(), String> {
        let stripped = strip_line_comment_preserving_strings(
            "let ch = '/'; let url = \"http://example.com\"; // trailing comment",
        );
        assert_eq!(
            stripped,
            "let ch = '/'; let url = \"http://example.com\"; "
        );

        let stripped_raw = strip_line_comment_preserving_strings(
            "let raw = r#\"http://example.com // keep\"#; // strip",
        );
        assert_eq!(stripped_raw, "let raw = r#\"http://example.com // keep\"#; ");

        let mut brace_depth: usize = 0;
        let mut paren_depth: usize = 0;
        update_ast_delimiter_depths(
            "if ch == '{' || ch == '(' || ch == '↑' {",
            &mut brace_depth,
            &mut paren_depth,
        );
        assert_eq!(brace_depth, 1);
        assert_eq!(paren_depth, 0);

        let mut brace_depth_interp: usize = 0;
        let mut paren_depth_interp: usize = 0;
        update_ast_delimiter_depths(
            "let msg = \"unclosed ${ ( in rust\"; if ready {",
            &mut brace_depth_interp,
            &mut paren_depth_interp,
        );
        assert_eq!(brace_depth_interp, 1);
        assert_eq!(paren_depth_interp, 0);

        let mut brace_depth_raw: usize = 0;
        let mut paren_depth_raw: usize = 0;
        update_ast_delimiter_depths(
            "let raw = r#\"{ ( [\"#; if ok {",
            &mut brace_depth_raw,
            &mut paren_depth_raw,
        );
        assert_eq!(brace_depth_raw, 1);
        assert_eq!(paren_depth_raw, 0);

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
            assert!(
                !contains_direct_range_slice(code),
                "Check 34 violation: direct range-slice indexing found at line {}: {}",
                line_idx.saturating_add(1),
                trimmed
            );
        }
        assert!(
            !prod_source.contains("#[allow(clippy::type_complexity)]"),
            "Check 34 violation: #[allow(clippy::type_complexity)] must be eliminated via type aliases"
        );
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
    /// - Panics or returns `Err(String)` if lexical state transitions or AST extraction fail.
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
    /// - Returns `Ok(())` when all scanner and AST extraction assertions pass.
    ///
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N) over the test fixture lines.
    /// - Space Complexity: O(N) for sanitized strings and extracted AST items.
    ///
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Fails immediately on state mismatch or missing AST items.
    ///
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-SOURCE-AST-SCAN-RUST-LINE-LEX-STATE], [REQ-SOURCE-AST-RUST-SCANNER-STATE]
    #[test]
    fn test_scan_rust_line_preserving_lex_state_char_literals_and_lifetimes() -> Result<(), String> {
        let mut state = RustStringScannerState::Normal;
        let line = "pub fn pick<'lifetime_alpha, 'lifetime_beta>(x: &'lifetime_alpha str, c: char) -> bool where 'lifetime_alpha: 'lifetime_beta { c == '{' || c == '\\'' || c == '\\u{1F600}' }";
        let sanitized = scan_rust_line_preserving_lex_state(line, &mut state);
        assert_eq!(state, RustStringScannerState::Normal);
        assert!(sanitized.contains("pub fn pick<'lifetime_alpha, 'lifetime_beta>"));
        assert!(!sanitized.contains("'{'"));
        assert!(!sanitized.contains("1F600"));

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
        assert!(items.iter().any(|i| i.name == "AfterComment"));
        assert!(items.iter().any(|i| i.name == "with_multi_hash_raw"));
        assert!(!items.iter().any(|i| i.name == "FakeInsideRaw"));
        Ok(())
    }

    /// Verifies semantic type aliases `ExtractedMethodSignature` and `ComponentMethodTable` for REQ-0601 AC-04.
    /// ## 1. Safety Intent & Regulatory Scope
    /// Conforms to DO-178C Level A and Check 34/Check 36 verification for semantic type aliases (`REQ-0601`).
    /// ## 2. Invariants & Mathematical Properties
    /// - `extract_method_signature` returns `None` for non-callable `Struct` AST items and `Some` for callable `Method` items.
    /// - `extract_component_method_table` and `lower_source_ast_to_package` preserve normalized parameters and return types.
    /// ## 3. Preconditions
    /// - Executed by `cargo test -p deap-core` on an abstract Rust component fixture (`ComponentAlpha`).
    /// ## 4. Postconditions
    /// - Negative probe (`Struct` -> `None`), `ComponentMethodTable` lookup, and `PartDef` action lowering assertions pass without indexing or `unwrap()`.
    /// ## 5. Algorithmic Complexity Bounds
    /// - Time Complexity: O(N log C) over parsed AST items. Space Complexity: O(N) fixture allocation.
    /// ## 6. Failure Modes & Fault Tolerance
    /// - Returns `Err(String)` or fails test execution immediately if signature extraction or table lookup deviates.
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
        let items = parse_rust_source_ast(rust_src, "src/component_alpha.rs")?;
        let struct_item = items
            .iter()
            .find(|i| i.name == "ComponentAlpha")
            .ok_or_else(|| "Missing ComponentAlpha struct AST item".to_string())?;
        assert_eq!(extract_method_signature(struct_item), None);

        let method_item = items
            .iter()
            .find(|i| i.name == "process_signal")
            .ok_or_else(|| "Missing process_signal method AST item".to_string())?;
        let extracted: ExtractedMethodSignature = extract_method_signature(method_item)
            .ok_or_else(|| "Expected Some(ExtractedMethodSignature) for process_signal".to_string())?;
        assert_eq!(
            extracted.0,
            vec![("input_val".to_string(), "Integer".to_string())]
        );
        assert_eq!(
            extracted.1,
            vec![("result".to_string(), "Boolean".to_string())]
        );
        assert_eq!(extracted.2, Some("Boolean".to_string()));

        let table: ComponentMethodTable = extract_component_method_table(&items);
        let methods = table
            .get("ComponentAlpha")
            .ok_or_else(|| "Missing ComponentAlpha entry in ComponentMethodTable".to_string())?;
        assert_eq!(methods.len(), 1);
        assert_eq!(methods.first(), Some(&extracted));

        let pkg = lower_source_ast_to_package(&items, "PkgAlpha");
        let part = pkg
            .part_defs
            .iter()
            .find(|p| p.name == "ComponentAlpha")
            .ok_or_else(|| "Missing ComponentAlpha PartDef in PkgAlpha".to_string())?;
        assert!(part.actions.iter().any(|a| {
            a.name == "process_signal" && a.in_params.len() == 1 && a.out_params.len() == 1
        }));
        Ok(())
    }
}

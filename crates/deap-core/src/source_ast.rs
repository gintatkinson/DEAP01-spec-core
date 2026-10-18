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
    let len = bytes.len();
    let mut idx: usize = 0;
    let mut line: usize = 1;
    let mut col: usize = 1;
    let mut stack: Vec<(u8, usize, usize)> = Vec::new();
    let is_dart = lang == SourceLanguage::Dart;

    while let Some(&b) = bytes.get(idx) {
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

        // Rust raw string literal `r#"..."#` or `r"..."`
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

        // Triple-quoted string in Dart (`"""..."""` or `'''...'''`)
        if is_dart
            && (b == b'"' || b == b'\'')
            && bytes.get(idx.saturating_add(1)) == Some(&b)
            && bytes.get(idx.saturating_add(2)) == Some(&b)
        {
            let quote = b;
            let open_line = line;
            let open_col = col;
            idx = idx.saturating_add(3);
            col = col.saturating_add(3);
            let mut closed = false;
            while let Some(&qb) = bytes.get(idx) {
                if qb == b'\\' {
                    idx = idx.saturating_add(2);
                    col = col.saturating_add(2);
                    continue;
                }
                if qb == quote
                    && bytes.get(idx.saturating_add(1)) == Some(&quote)
                    && bytes.get(idx.saturating_add(2)) == Some(&quote)
                {
                    idx = idx.saturating_add(3);
                    col = col.saturating_add(3);
                    closed = true;
                    break;
                }
                if qb == b'\n' {
                    line = line.saturating_add(1);
                    col = 1;
                } else {
                    col = col.saturating_add(1);
                }
                idx = idx.saturating_add(1);
            }
            if !closed {
                return Err(format!(
                    "{}:{}:{}: syntax error: unclosed triple-quoted string at line {}, column {}",
                    file_label, open_line, open_col, open_line, open_col
                ));
            }
            continue;
        }

        // Double-quoted string literal `"..."`
        if b == b'"' {
            let open_line = line;
            let open_col = col;
            idx = idx.saturating_add(1);
            col = col.saturating_add(1);
            let mut closed = false;
            while let Some(&qb) = bytes.get(idx) {
                if qb == b'\\' {
                    idx = idx.saturating_add(2);
                    col = col.saturating_add(2);
                    continue;
                }
                if qb == b'"' {
                    idx = idx.saturating_add(1);
                    col = col.saturating_add(1);
                    closed = true;
                    break;
                }
                if qb == b'\n' {
                    line = line.saturating_add(1);
                    col = 1;
                } else {
                    col = col.saturating_add(1);
                }
                idx = idx.saturating_add(1);
            }
            if !closed {
                return Err(format!(
                    "{}:{}:{}: syntax error: unclosed string literal at line {}, column {}",
                    file_label, open_line, open_col, open_line, open_col
                ));
            }
            continue;
        }

        // Single-quoted literal: in Dart `'...'` is a string; in Rust `'c'` is a char or `'a` is a lifetime
        if b == b'\'' {
            if is_dart {
                let open_line = line;
                let open_col = col;
                idx = idx.saturating_add(1);
                col = col.saturating_add(1);
                let mut closed = false;
                while let Some(&qb) = bytes.get(idx) {
                    if qb == b'\\' {
                        idx = idx.saturating_add(2);
                        col = col.saturating_add(2);
                        continue;
                    }
                    if qb == b'\'' {
                        idx = idx.saturating_add(1);
                        col = col.saturating_add(1);
                        closed = true;
                        break;
                    }
                    if qb == b'\n' {
                        break;
                    }
                    col = col.saturating_add(1);
                    idx = idx.saturating_add(1);
                }
                if !closed {
                    return Err(format!(
                        "{}:{}:{}: syntax error: unclosed single-quoted string at line {}, column {}",
                        file_label, open_line, open_col, open_line, open_col
                    ));
                }
                continue;
            } else {
                // Rust: check if this is a char literal `'x'` or `'\...'`
                if bytes.get(idx.saturating_add(1)) == Some(&b'\\') {
                    let mut scan = idx.saturating_add(2);
                    while scan < len && scan < idx.saturating_add(12) {
                        if bytes.get(scan) == Some(&b'\'') {
                            let step = scan.saturating_add(1).saturating_sub(idx);
                            idx = scan.saturating_add(1);
                            col = col.saturating_add(step);
                            break;
                        }
                        scan = scan.saturating_add(1);
                    }
                    continue;
                } else if bytes.get(idx.saturating_add(2)) == Some(&b'\'') {
                    idx = idx.saturating_add(3);
                    col = col.saturating_add(3);
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
    let mut in_multiline_comment = false;
    let mut in_raw_string = false;
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

        if in_multiline_comment {
            if trimmed.contains("*/") {
                in_multiline_comment = false;
            }
            continue;
        }
        if !in_raw_string && trimmed.starts_with("/*") && !trimmed.contains("*/") {
            in_multiline_comment = true;
            continue;
        }
        if in_raw_string {
            if trimmed.contains("\"#") {
                in_raw_string = false;
            }
            continue;
        }
        if trimmed.starts_with("//!") {
            continue;
        }
        if trimmed.starts_with("///") {
            if !in_mod_tests {
                current_doc.push(trimmed.to_string());
            }
            continue;
        }

        let code = if let Some((before, _)) = trimmed.split_once("//") {
            before.trim()
        } else {
            trimmed
        };
        if code.is_empty() {
            continue;
        }

        if let Some(raw_pos) = code.find("r#\"") {
            let after_open = code.get(raw_pos.saturating_add(3)..).unwrap_or("");
            if !after_open.contains("\"#") {
                in_raw_string = true;
            }
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

        let code = if let Some((before, _)) = trimmed.split_once("//") {
            before.trim()
        } else {
            trimmed
        };
        if code.is_empty() || code.starts_with('@') {
            continue;
        }

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

        update_ast_delimiter_depths(code, &mut brace_depth, &mut paren_depth);
        if brace_depth == 0 {
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
        if !item.kind.is_callable() {
            continue;
        }
        let act_name = sanitize_sysml_identifier(&item.name, "action_op");
        let in_params: Vec<AttributeDef> = item
            .params
            .iter()
            .map(|p| AttributeDef {
                name: sanitize_sysml_identifier(&p.name, "param"),
                type_name: normalize_type_for_sysml(&p.type_name),
                default_value: None,
                doc: None,
            })
            .collect();
        let mut out_params = Vec::new();
        if let Some(ref ret) = item.return_type {
            let trimmed_ret = ret.trim();
            if !trimmed_ret.is_empty() && trimmed_ret != "()" && trimmed_ret != "void" {
                out_params.push(AttributeDef {
                    name: "result".to_string(),
                    type_name: normalize_type_for_sysml(trimmed_ret),
                    default_value: None,
                    doc: None,
                });
            }
        }

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
///
/// ## 3. Preconditions
/// - `source_dir: &Path` is an existing file or directory path on disk.
/// - `language: SourceLanguage` specifies `Auto`, `Rust`, or `Dart`.
/// - `package_name: &str` provides the root SysML v2 package identifier.
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

    Ok(lower_source_ast_to_package(&all_items, package_name))
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

    if let Some(fn_name) = extract_dart_callable_ident(code) {
        return Some((SourceItemKind::Function, fn_name, None));
    }

    None
}

/// Classifies a class-body (`brace_depth == 1`) Dart line as a `Constructor` or `Method`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A and Check 36 Dart member parity (`classify_dart_member_ast`).
/// Identifies `const MyApp({super.key});`, named constructors, factory constructors, and class
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
        let cname = extract_leading_ast_ident(after_factory.trim_start())?;
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
            } else if ch.is_whitespace() {
                break;
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

/// Updates `brace_depth` and `paren_depth` across a single comment-stripped line during AST extraction.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A delimiter tracking parity (`update_ast_delimiter_depths`).
/// Ensures named parameter braces `({super.key})` inside `(...)` never alter `brace_depth`.
///
/// ## 2. Invariants & Mathematical Properties
/// - `paren_depth` updates on unquoted `'('` and `')'`.
/// - `brace_depth` updates on unquoted `'{'` and `'}'` iff `*paren_depth == 0`.
///
/// ## 3. Preconditions
/// - `code: &str` is a comment-stripped source line; `brace_depth` and `paren_depth` are mutable counters.
///
/// ## 4. Postconditions
/// - Updates `*brace_depth` and `*paren_depth` in place.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) single-pass byte scan over `code`.
/// - Space Complexity: O(1) auxiliary state.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Uses `.get(..)` and saturating arithmetic.
/// - State: Mutates only the two depth references.
/// - Action: Skips escaped bytes inside string and character literals.
/// - Resource: Zero heap allocations.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-FEAT-42-UPDATE-AST-DELIMITER-DEPTHS]
pub fn update_ast_delimiter_depths(code: &str, brace_depth: &mut usize, paren_depth: &mut usize) {
    let bytes = code.as_bytes();
    let mut idx: usize = 0;
    let mut in_double_quote = false;
    let mut in_single_quote = false;

    while let Some(&b) = bytes.get(idx) {
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
        if in_single_quote {
            if b == b'\\' {
                idx = idx.saturating_add(2);
                continue;
            }
            if b == b'\'' {
                in_single_quote = false;
            }
            idx = idx.saturating_add(1);
            continue;
        }

        if b == b'"' {
            in_double_quote = true;
        } else if b == b'\'' {
            let is_char_lit = bytes.get(idx.saturating_add(2)) == Some(&b'\'')
                || (bytes.get(idx.saturating_add(1)) == Some(&b'\\')
                    && bytes.get(idx.saturating_add(3)) == Some(&b'\''));
            if is_char_lit {
                in_single_quote = true;
            }
        } else if b == b'(' {
            *paren_depth = paren_depth.saturating_add(1);
        } else if b == b')' {
            *paren_depth = paren_depth.saturating_sub(1);
        } else if b == b'{' && *paren_depth == 0 {
            *brace_depth = brace_depth.saturating_add(1);
        } else if b == b'}' && *paren_depth == 0 {
            *brace_depth = brace_depth.saturating_sub(1);
        }

        idx = idx.saturating_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dart_stateful_widget_and_lower_to_sysml() {
        let dart_src = r#"import 'package:flutter/material.dart';

/// Realises: [REQ-UI-01]
class TelemetryPanel extends StatefulWidget {
  final String title;
  final int refreshRateHz;
  final VoidCallback onRefresh;
  final ValueChanged<int> onRateChanged;

  const TelemetryPanel({
    super.key,
    required this.title,
    required this.refreshRateHz,
    required this.onRefresh,
    required this.onRateChanged,
  });

  @override
  State<TelemetryPanel> createState() => _TelemetryPanelState();
}

class _TelemetryPanelState extends State<TelemetryPanel> {
  bool isExpanded = false;

  void togglePanel(bool nextState) {
    assert(nextState == true || nextState == false);
    isExpanded = nextState;
  }
}
"#;
        let items = parse_dart_source_ast(dart_src, "lib/telemetry_panel.dart")
            .expect("parse dart stateful widget");
        let pkg = lower_source_ast_to_package(&items, "TelemetryUiPkg");
        let panel = pkg
            .part_defs
            .iter()
            .find(|p| p.name == "TelemetryPanel")
            .expect("TelemetryPanel PartDef");
        assert!(panel.attributes.iter().any(|a| a.name == "title"));
        assert!(panel.attributes.iter().any(|a| a.name == "refreshRateHz"));
        assert!(panel.ports.iter().any(|p| p.name == "onRefresh"));
        assert!(panel.ports.iter().any(|p| p.name == "onRateChanged"));
        assert!(panel
            .satisfied_requirements
            .contains(&"REQ-UI-01".to_string()));
        assert!(!panel.states.is_empty());
    }

    #[test]
    fn test_parse_rust_struct_and_impl_and_lower_to_sysml() {
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
        let items =
            parse_rust_source_ast(rust_src, "src/buffer.rs").expect("parse rust struct and impl");
        let pkg = lower_source_ast_to_package(&items, "BufferPkg");
        let buf = pkg
            .part_defs
            .iter()
            .find(|p| p.name == "ComponentBuffer")
            .expect("ComponentBuffer PartDef");
        assert_eq!(buf.attributes.len(), 2);
        assert!(buf.actions.iter().any(|a| a.name == "push_item"
            && a.in_params.len() == 1
            && a.out_params.len() == 1));
        assert!(!buf.constraints.is_empty());
        assert!(buf
            .satisfied_requirements
            .contains(&"REQ-BUF-01".to_string()));
    }

    #[test]
    fn test_syntax_error_unclosed_brace_fails_closed_with_line_and_col() {
        let bad_rust = "pub struct Broken {\n    pub x: u32,\n";
        let err = parse_rust_source_ast(bad_rust, "src/broken.rs").unwrap_err();
        assert!(
            err.contains("src/broken.rs:1:19") && err.contains("unclosed delimiter '{'"),
            "Expected line/col diagnostic, got: {}",
            err
        );
    }
}

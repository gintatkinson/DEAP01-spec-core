//! SysML v2 / KerML compiler and specification synchronization engine.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Provides the primary library facade for deterministic lexical analysis, syntactic parsing,
//! semantic validation, STPA/FMECA hazard synthesis, and bidirectional Markdown synchronization
//! under DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D tool qualification standards.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Deterministic AST construction: identical UTF-8 input streams always yield structurally
//!   identical `PackageDef` syntax trees and cryptographic schema digests.
//! - Zero hardcoded domain concepts: all exported compiler primitives operate exclusively over
//!   abstract SysML v2 / KerML meta-model structures.
//!
//! ## 3. Preconditions
//! - Input SysML v2 source slices must be valid UTF-8 strings representing textual model
//!   declarations or workspace schema trees.
//!
//! ## 4. Postconditions
//! - Exported parser and synchronization entry points return strongly typed AST structures or
//!   explicit, span-annotated diagnostic errors without panicking or mutating global state.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Top-level parsing via `parse_sysml` operates in $O(N)$ time and $O(N)$ memory with respect
//!   to the byte length $N$ of the input source buffer.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Lexical or grammatical syntax violations are captured as deterministic `ParseError` values
//!   carrying exact line, column, and byte span provenance.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-COMPILER-SYSML-LIB], [DO-330-TQL-4-LIB]

pub mod docstring;
pub mod lexer;
pub mod parser;
pub mod semantic;
pub mod stpa;
pub mod sync;

pub use deap_core::sysml_ast::*;
pub use docstring::{
    inject_dart_file_docstrings, inject_rust_file_docstrings, run_docstring_injection,
    run_extract_source, DocstringExitCode, DocstringInjectionOptions, DocstringInjectionReport,
};
pub use lexer::scanner::Scanner;
pub use lexer::token::{Span, Token, TokenKind};
pub use parser::expressions::{parse_expression, BinaryOp, Expr, ExpressionParser, UnaryOp};
pub use parser::grammar::{
    discover_sysml_files, merge_package_defs, parse_sysml_directory, parse_sysml_tree, ParseError,
    SysmlParser,
};
pub use semantic::digest::{
    compute_file_manifest, count_structural_elements, generate_digest, write_atomic,
    write_digest_atomic, FileDigestEntry, SchemaDigest,
};
pub use semantic::serializer::{to_sysml, SysmlSerializable};
pub use semantic::symbols::{Symbol, SymbolKind, SymbolTable};
pub use semantic::validator::{DiagnosticSeverity, SemanticDiagnostic, SemanticValidator};
pub use stpa::{
    compile_stpa_to_constraints, compile_stpa_to_constraints_filtered, emit_safety_suite, expand_cartesian_stpa, generate_fmeca_matrix,
    transpile_safety_suite, transpile_stpa, FmecaRow, FmecaScoringConfig, UnsafeControlAction,
};
pub use sync::{
    forward_sync_sysml_to_specs, reverse_sync_specs_to_sysml, ForwardSyncOptions,
    ReverseSyncOptions,
};

/// Parse textual SysML v2 source into a strongly-typed PackageDef AST.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Serves as the canonical single-buffer entry point for parsing textual SysML v2 specifications
/// into an in-memory `PackageDef` abstract syntax tree under DO-330 TQL-4 determinism mandates.
///
/// ## 2. Invariants & Mathematical Properties
/// - Total function over `&str`: never panics or invokes undefined behavior on arbitrary input.
/// - Pure functional transformation: produces an isolated `PackageDef` root named `"SysML_Model"`.
///
/// ## 3. Preconditions
/// - `source` is an in-memory UTF-8 string slice containing SysML v2 declarations.
///
/// ## 4. Postconditions
/// - Returns `Ok(PackageDef)` containing all parsed child elements in declaration order when
///   `source` conforms to the grammar, or `Err(ParseError)` on the first syntax error.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time complexity: $O(N)$ where $N$ is `source.len()`.
/// - Space complexity: $O(N)$ proportional to token stream and AST node count.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Malformed tokens, unclosed delimiters, or unexpected keywords return `Err(ParseError)` with
///   source span coordinates.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-COMPILER-SYSML-PARSE]
pub fn parse_sysml(source: &str) -> Result<PackageDef, ParseError> {
    SysmlParser::parse_source(source, "SysML_Model")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_sysml_lib_7_section_rustdoc() {
        let src = include_str!("lib.rs");
        let module_doc: String = src
            .lines()
            .take_while(|line| line.starts_with("//!"))
            .collect::<Vec<&str>>()
            .join("\n");

        assert!(module_doc.contains("//! ## 1. Safety Intent & Regulatory Scope"));
        assert!(module_doc.contains("//! ## 2. Invariants & Mathematical Properties"));
        assert!(module_doc.contains("//! ## 3. Preconditions"));
        assert!(module_doc.contains("//! ## 4. Postconditions"));
        assert!(module_doc.contains("//! ## 5. Algorithmic Complexity Bounds"));
        assert!(module_doc.contains("//! ## 6. Failure Modes & Fault Tolerance"));
        assert!(module_doc.contains("//! ## 7. Realization & Traceability"));
        assert!(module_doc.contains("//! Realises: [REQ-COMPILER-SYSML-LIB], [DO-330-TQL-4-LIB]"));

        assert!(src.contains("/// ## 1. Safety Intent & Regulatory Scope"));
        assert!(src.contains("/// ## 2. Invariants & Mathematical Properties"));
        assert!(src.contains("/// ## 3. Preconditions"));
        assert!(src.contains("/// ## 4. Postconditions"));
        assert!(src.contains("/// ## 5. Algorithmic Complexity Bounds"));
        assert!(src.contains("/// ## 6. Failure Modes & Fault Tolerance"));
        assert!(src.contains("/// ## 7. Realization & Traceability"));
        assert!(src.contains("/// Realises: [REQ-COMPILER-SYSML-PARSE]"));

        let parsed = parse_sysml("package Demo {}");
        assert!(parsed.is_ok());
    }
}

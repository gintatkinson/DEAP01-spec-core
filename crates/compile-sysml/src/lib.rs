//! SysML v2 / KerML compiler and specification synchronization engine.

pub mod lexer;
pub mod parser;
pub mod semantic;
pub mod stpa;
pub mod sync;

pub use deap_core::sysml_ast::*;
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
    compile_stpa_to_constraints, emit_safety_suite, expand_cartesian_stpa, generate_fmeca_matrix,
    transpile_safety_suite, transpile_stpa, FmecaRow, FmecaScoringConfig, UnsafeControlAction,
};
pub use sync::{
    forward_sync_sysml_to_specs, reverse_sync_specs_to_sysml, ForwardSyncOptions,
    ReverseSyncOptions,
};

/// Parse textual SysML v2 source into a strongly-typed PackageDef AST.
pub fn parse_sysml(source: &str) -> Result<PackageDef, ParseError> {
    SysmlParser::parse_source(source, "SysML_Model")
}


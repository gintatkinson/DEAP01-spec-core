//! Semantic analysis, validation, serialization, and digest generator module.

pub mod digest;
pub mod serializer;
pub mod symbols;
pub mod validator;

pub use digest::SchemaDigest;
pub use serializer::SysmlSerializable;
pub use symbols::{Symbol, SymbolKind, SymbolTable};
pub use validator::{DiagnosticSeverity, SemanticDiagnostic, SemanticValidator};

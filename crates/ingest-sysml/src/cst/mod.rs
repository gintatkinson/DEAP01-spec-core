//! # Concrete Syntax Tree (`logos` + `rowan` + `pulldown-cmark` + `serde_json`) Translators (`[DOC-TIER: A]`)
//!
//! 1. **Purpose**: Provides lossless CST and typed AST frontends lowering OMG IDL, IETF YANG,
//!    OpenAPI 3.0/3.1, Level 0 OEM Markdown, and Rust/Dart source trees directly into canonical
//!    SysML v2 HIR/AST models (`deap_core::PackageDef`).
//! 2. **Scope**: Consolidates `idl_cst`, `yang_cst`, `markdown_cst`, `openapi_ast`, and `code_cst`.
//! 3. **Invariants**: Zero raw string-rewriting translators; deterministic `BTreeMap`/`BTreeSet` lowering.
//! 4. **Concurrency**: Pure, stateless translator implementations (`Send + Sync`).
//! 5. **Error Handling**: Fail-closed `Result<PackageDef, String>` propagation without panics.
//! 6. **Security & Safety**: Zero `unsafe`, zero unchecked slice indexing (`[..]`).
//! 7. **Traceability**: Realises: `[REQ-0001]`, `[REQ-0015]`, `[REQ-0021]`, `[REQ-0201]`, `[REQ-SYSML-INGEST-OAS]`, `[REQ-FEAT-42-CODE-TRANSLATOR]`.

#![deny(clippy::all, clippy::pedantic, clippy::nursery)]

pub mod code_cst;
pub mod idl_cst;
pub mod markdown_cst;
pub mod md_behavioral;
pub mod md_common;
pub mod md_parametric;
pub mod md_requirements;
pub mod md_tables;
pub mod openapi_ast;
pub mod yang_cst;
pub mod yang_lower;

pub use code_cst as code;
pub use idl_cst as idl;
pub use markdown_cst as markdown;
pub use openapi_ast as openapi;
pub use yang_cst as yang;

pub use code_cst::CodeTranslator;
pub use idl_cst::IDLTranslator;
pub use markdown_cst::MarkdownTranslator;
pub use openapi_ast::OpenAPITranslator;
pub use yang_cst::YangTranslator;

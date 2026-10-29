//! Target-language emitter dispatchers for modular SysML v2 code synthesis.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Isolates language-specific contract emitters (Rust and Flutter/Dart) behind
//! deterministic, schema-derived code generation boundaries with zero domain hardcoding.
//!
//! ## 2. Architectural Role & Boundaries
//! Aggregates target emitter submodules under `modular_codegen::emitters` and
//! re-exports canonical Rust modular tree synthesis entrypoints for compiler callers.
//!
//! ## 3. Data Contracts & Invariants
//! All submodules consume validated modular IR / AST representations and produce
//! byte-deterministic source units with workspace-relative paths and explicit manifests.
//!
//! ## 4. Failure Modes & Mitigation
//! Filesystem emission and path resolution errors surface via explicit `Result` types
//! without panics, silent truncation, or partial unverified state mutations.
//!
//! ## 5. Determinism & Concurrency
//! Pure synthesis helpers operate over immutable inputs in deterministic lexical order
//! with zero shared mutable global state.
//!
//! ## 6. Verification & Test Strategy
//! Verified by crate-level unit and mutation tests exercising modular Rust tree
//! synthesis, manifest hashing, and Flutter contract emission invariants.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0607], [REQ-EMIT-FLUTTER-CONTRACTS]

pub mod flutter;
pub mod rust_emitter;

pub use rust_emitter::{
    compute_directory_tree_manifest, emit_modular_rust_tree, synthesize_modular_rust_files,
    ModularRustOptions, SynthesizedRustUnit,
};

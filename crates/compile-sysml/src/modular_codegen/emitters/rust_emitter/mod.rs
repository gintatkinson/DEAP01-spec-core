//! Layer 3 deterministic modular Rust source emitter, bounds auditor, and tree manifest builder.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Provides deterministic Rust code emission for modular SysML v2 AST partitions under DO-178C Level A.
//!
//! ## 2. Invariants & Mathematical Properties
//! File line count <= 380, function length <= 80 lines, arity <= 5 parameters.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0607]

pub mod manifest;
pub mod render;

pub use manifest::*;
pub use render::*;

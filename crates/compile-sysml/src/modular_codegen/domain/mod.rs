//! Modular codegen Layer 1 domain model barrel facade.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Exposes immutable value types for structural modularity bounds, layer roles,
//! and multi-file compilation-unit dependency graphs under high-assurance MBSE
//! code generation and structural auditability constraints.
//!
//! ## 2. Inputs & Preconditions
//! Consumed as pure type definitions by higher modular codegen layers -- imposes
//! zero runtime filesystem or I/O preconditions.
//!
//! ## 3. Outputs & Postconditions
//! Re-exports deterministic value objects (`ModularityBounds`, `LayerRole`,
//! `ModuleGraph`, and associated plan/manifest types) with total `Eq` semantics.
//!
//! ## 4. Determinism & Side-Effect Isolation
//! Pure declarative facade containing zero mutable global state, I/O, or
//! non-deterministic collections.
//!
//! ## 5. Failure Modes & Diagnostics
//! Re-exports structured `ModularityErrorCode` and `ModularityDiagnostic` types
//! so callers surface deterministic violations without panicking.
//!
//! ## 6. Complexity & Resource Bounds
//! Zero runtime execution cost (`O(1)` compile-time symbol re-export facade;
//! file length strictly bounded `<= 45` lines).
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0607]

pub mod bounds;
pub mod layer_role;
pub mod module_graph;

pub use bounds::{ModularityBounds, ModularityDiagnostic, ModularityErrorCode};
pub use layer_role::LayerRole;
pub use module_graph::{
    CompilationUnit, DependencyEdge, GeneratedFileDigest, GeneratedTreeManifest,
    HoistedPredicate, ModuleGraph, OutlinedActionPlan, OutlinedActionStage,
    OutlinedStateHandler, OutlinedStateMachinePlan, UnitKind,
};

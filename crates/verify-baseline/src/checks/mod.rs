//! Baseline verification check modules (Checks 10 through 36).
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C baseline
//! verification architecture standards. Aggregates and re-exports all deterministic structural,
//! mathematical, Mermaid, STPA safety, WBS, parity, domain cleanliness, and Check 36 docstring
//! completeness gates.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Deterministic Re-Export Surface: Every baseline check `10..=36` is exposed via a stable
//!   module namespace under `verify_baseline::checks`.
//!
//! ## 3. Preconditions
//! - Invoked by `verify_baseline::runner` or integration test suites with valid workspace paths.
//!
//! ## 4. Postconditions
//! - Provides compile-time linked access to all baseline verification gate functions and outcomes.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: O(1) module linkage overhead.
//! - Space Complexity: O(1) static symbol table dispatch.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Propagates all submodule `Result` errors without silent suppression.
//! - State: Zero module-level mutable state.
//! - Action: Pure namespace aggregation.
//! - Resource: Zero runtime memory allocation.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-VERIFY-BASELINE-CHECKS-MOD], [DO-330-TQL-4-CHECKS-MOD]

pub mod docstring_gate;
pub mod domain_cleanliness_gate;
pub mod math_katex;
pub mod mermaid;
pub mod parity_gates;
pub mod safety_stpa;
pub mod structural;
pub mod wbs_integrity;

pub use docstring_gate::*;
pub use domain_cleanliness_gate::*;
pub use math_katex::check_latex_katex_syntax;
pub use mermaid::check_mermaid_syntax;
pub use parity_gates::*;
pub use safety_stpa::{check_safety_integrity, SafetyCheckOutcome};
pub use structural::*;
pub use wbs_integrity::{check_wbs_suite_integrity, WbsCheckOutcome};


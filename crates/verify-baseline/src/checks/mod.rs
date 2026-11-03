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

pub mod cargo_policy_gate;
pub mod clippy_ndjson_gate;
pub mod codebase;
pub mod codebase_gesture_gate;
pub mod codebase_mock_gate;
pub mod codebase_solver_gate;
pub mod cst_attr_gate;
pub mod cst_purity_gate;
#[cfg(test)]
mod codebase_tests_a;
#[cfg(test)]
mod codebase_tests_b;
pub(crate) mod doc_gate_dart;
pub(crate) mod doc_gate_rust;
pub(crate) mod doc_gate_rust_classify;
#[cfg(test)]
mod doc_gate_tests;
pub(crate) mod doc_gate_validate;
pub mod docstring_cst_gate;
pub mod docstring_gate;
pub mod docstring_tier_rules;
mod dod_ast_visitor;
pub mod dod_gates;
#[cfg(test)]
mod dod_gates_tests;
pub mod domain_cleanliness_gate;
mod domain_cleanliness_rules;
pub mod formal_stage2_dylint;
pub mod formal_stage3_4_miri_kani;
pub mod formal_stage5_ferrocene;
pub mod io_fail_closed_gate;
pub mod math_katex;
pub mod mermaid;
#[cfg(test)]
mod parity_check_a_tests;
#[cfg(test)]
mod parity_check_b_tests;
#[cfg(test)]
mod parity_check_c_tests;
pub(crate) mod parity_check_conops_exec;
pub(crate) mod parity_check_coverage;
pub(crate) mod parity_check_cross_doc;
#[cfg(test)]
mod parity_check_d_tests;
pub(crate) mod parity_check_discovery;
pub(crate) mod parity_check_dual_ssot;
#[cfg(test)]
mod parity_check_e_tests;
pub(crate) mod parity_check_grounding;
pub(crate) mod parity_check_harvest;
pub(crate) mod parity_check_icd_ast;
pub(crate) mod parity_check_icd_gate;
pub(crate) mod parity_check_icd_signals;
pub(crate) mod parity_check_obligation;
pub(crate) mod parity_check_ops_std;
pub(crate) mod parity_check_semantic;
pub(crate) mod parity_check_slicer;
pub mod parity_gates;
pub mod ratchet_table;
pub(crate) mod safety_fmeca;
pub(crate) mod safety_pillars;
pub mod safety_stpa;
pub mod wbs_integrity;

/// Compatibility namespace re-exporting CST purity, fail-closed I/O, and Cargo policy gates (`[REQ-0602]`).
pub mod structural {
    pub use super::cargo_policy_gate::*;
    pub use super::cst_purity_gate::*;
    pub use super::domain_cleanliness_gate::verify_upstream_blueprint_domain_cleanliness;
    pub use super::io_fail_closed_gate::*;
}

pub use cargo_policy_gate::*;
pub use codebase::*;
pub use cst_purity_gate::*;
pub use docstring_cst_gate::*;
pub use docstring_gate::*;
pub use dod_gates::{
    check_codegen_zero_concat, check_dod_gates, check_salsa_incremental_memoization,
    check_sysml_clause8_coverage, check_upstream_self_hosting_parity,
    check_zero_hand_rolled_lexers, check_zero_legacy_ratchet, Finding,
};
pub use domain_cleanliness_gate::*;
pub use io_fail_closed_gate::*;
pub use math_katex::check_latex_katex_syntax;
pub use mermaid::check_mermaid_syntax;
pub use parity_gates::*;
pub use ratchet_table::{lookup_ratchet_ceiling, LEGACY_RATCHET_MAX_LINES};
pub use safety_stpa::{check_safety_integrity, SafetyCheckOutcome};
pub use wbs_integrity::{check_wbs_suite_integrity, WbsCheckOutcome};

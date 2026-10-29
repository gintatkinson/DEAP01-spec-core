//! Unified integration test harness for `compile-sysml`.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C software
//! verification standards. Consolidates all `compile-sysml` integration test suites into a single
//! linked test binary while preserving exact on-disk `tests/<stem>.rs` file paths for defect
//! dossier and upstream projection traceability gates.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Every existing `crates/compile-sysml/tests/<stem>.rs` module remains at its canonical path
//!   and is mounted via `pub mod <stem>;` under a single `[[test]] name = "integration"` target.
//! - Scopes `#[cfg_attr(test, allow(clippy::field_reassign_with_default))]` strictly as an outer
//!   module attribute on `pub mod flutter_codegen_tests;`.
//!
//! ## 3. Preconditions
//! - `compile-sysml` library and binary (`CARGO_BIN_EXE_compile-sysml`) compile cleanly under
//!   workspace `forbid` lints.
//!
//! ## 4. Postconditions
//! - Executes 100% of `docstring_injector_tests`, `flutter_codegen_tests`,
//!   `multi_file_compilation_tests`, `reproducer_tests`, and
//!   `upstream_projection_acceptance_test` within one deterministic test runner process.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(T)$ across $T$ total registered `#[test]` cases with a single link pass.
//! - Space Complexity: $O(1)$ module dispatch overhead.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Preserves `CARGO_BIN_EXE_compile-sysml` subprocess isolation per test case.
//! - State: Each test allocates isolated temporary directories removed on `Drop`.
//! - Action: Fails closed on any assertion failure across mounted submodules.
//! - Resource: Eliminates 4 redundant integration binary link passes.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-COMPILE-SYSML-UNIFIED-INTEGRATION-HARNESS]

pub mod docstring_injector_tests;
#[cfg_attr(test, allow(clippy::field_reassign_with_default))]
pub mod flutter_codegen_tests;
pub mod multi_file_compilation_tests;
pub mod reproducer_tests;
pub mod upstream_projection_acceptance_test;

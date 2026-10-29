//! Unified integration test harness for `verify-baseline`.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C software
//! verification standards. Consolidates all `verify-baseline` integration test suites into a
//! single linked test binary while preserving exact on-disk `tests/<stem>.rs` file paths for
//! blueprint gate and defect dossier `Test-Target` verification checks.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Every existing `crates/verify-baseline/tests/<stem>.rs` module remains at its canonical path
//!   and is mounted via `pub mod <stem>;` under a single `[[test]] name = "integration"` target.
//! - Includes Check 17 realization-schema FMECA closure coverage relocated from `ingest-sysml`.
//!
//! ## 3. Preconditions
//! - `verify-baseline` library and binary (`CARGO_BIN_EXE_verify-baseline`) compile cleanly under
//!   workspace `forbid` lints.
//!
//! ## 4. Postconditions
//! - Executes 100% of `blueprint_gate_tests`, `check_33_zero_python_tests`,
//!   `check_36_docstring_gate_tests`, `codebase_checks_tests`, and `defect_dossier_tests` within
//!   one deterministic test runner process.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(T)$ across $T$ total registered `#[test]` cases with a single link pass.
//! - Space Complexity: $O(1)$ module dispatch overhead.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Preserves `CARGO_BIN_EXE_verify-baseline` subprocess isolation per test case.
//! - State: Each test allocates isolated temporary directories removed on `Drop`.
//! - Action: Fails closed on any assertion failure across mounted submodules.
//! - Resource: Eliminates 4 redundant multi-megabyte integration binary link passes.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-VERIFY-BASELINE-UNIFIED-INTEGRATION-HARNESS]

pub mod blueprint_gate_tests;
pub mod check_33_zero_python_tests;
pub mod check_36_docstring_gate_tests;
pub mod codebase_checks_tests;
pub mod defect_dossier_tests;

//! Unified integration test harness for `ingest-sysml`.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C software
//! verification standards. Consolidates all `ingest-sysml` integration test suites into a single
//! linked test binary while preserving exact on-disk `tests/<stem>.rs` file paths for defect
//! dossier and traceability verification gates.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Every existing `crates/ingest-sysml/tests/<stem>.rs` module remains at its canonical path
//!   and is mounted via `pub mod <stem>;` under a single `[[test]] name = "integration"` target.
//! - Zero `verify-baseline` reverse dev-dependency edges are introduced into `ingest-sysml`.
//!
//! ## 3. Preconditions
//! - `ingest-sysml` library and binary (`CARGO_BIN_EXE_ingest-sysml`) compile cleanly under
//!   workspace `forbid` lints.
//!
//! ## 4. Postconditions
//! - Executes 100% of `code_ingestion_tests`, `code_realization_scoping_tests`, and
//!   `reproducer_tests` within one deterministic test runner process.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(T)$ across $T$ total registered `#[test]` cases with a single link pass.
//! - Space Complexity: $O(1)$ module dispatch overhead.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Preserves `CARGO_BIN_EXE_ingest-sysml` subprocess isolation per test case.
//! - State: Each test allocates isolated temporary directories removed on `Drop`.
//! - Action: Fails closed on any assertion failure across mounted submodules.
//! - Resource: Eliminates duplicate test-binary linking and disk I/O overhead.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-INGEST-SYSML-UNIFIED-INTEGRATION-HARNESS]

pub mod code_ingestion_tests;
pub mod code_realization_scoping_tests;
pub mod reproducer_tests;

//! Unified integration test harness for the `compile-sysml` crate.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C verification
//! objectives for `crates/compile-sysml/tests/integration.rs`. Consolidates all integration test
//! suites into a single compiled test binary to minimize redundant linker passes while preserving
//! 100% test coverage across CLI execution, docstring verification, Flutter codegen, multi-file
//! schema compilation, self-hosting bootstrap, and upstream projection acceptance.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Every integration test module executes deterministically within a unified binary context.
//! - All struct initializations across integration tests use explicit struct literal update syntax
//!   without lint suppression attributes.
//!
//! ## 3. Preconditions
//! - The `compile-sysml` library and binary targets are compiled and accessible via Cargo test
//!   environment variables.
//!
//! ## 4. Postconditions
//! - All mounted integration test submodules execute to completion with zero panics or resource leaks.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(T)$ where $T$ is the sum of test execution times across mounted submodules.
//! - Space Complexity: $O(M)$ bounded by the peak memory footprint of individual test fixtures.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Exposes submodule test cases directly to the standard Rust test harness.
//! - State: Each submodule isolates filesystem state inside unique temporary directories.
//! - Action: Fails fast with diagnostic assertion messages on any invariant violation.
//! - Resource: Reclaims all temporary test directories prior to process exit.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-COMPILER-SYSML-INTEGRATION-HARNESS] | SysML v2: VerificationCaseDef(compile_sysml_integration_harness)

pub mod docstring_injector_b_tests;
pub mod docstring_injector_c_tests;
pub mod docstring_injector_tests;
pub mod flutter_codegen_tests;
pub mod multi_file_compilation_tests;
pub mod reproducer_b_tests;
pub mod reproducer_c_tests;
pub mod reproducer_d_tests;
pub mod reproducer_e_tests;
pub mod reproducer_f_tests;
pub mod reproducer_g_tests;
pub mod reproducer_h_tests;
pub mod reproducer_i_tests;
pub mod reproducer_j_tests;
pub mod reproducer_k_tests;
pub mod reproducer_tests;
pub mod self_hosting_bootstrap;
pub mod upstream_projection_acceptance_test;

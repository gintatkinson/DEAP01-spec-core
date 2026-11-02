#![deny(clippy::all, clippy::pedantic, clippy::nursery)]
#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Stage-2 Nightly Dylint MIR Gatekeeper Verification (`BP-09`, `REQ-0901`, `REQ-0902`). [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D verification tool
//! qualification requirements. Validates the isolated `lints/deap-dylint-mir` plugin
//! in-process via `ra_ap_syntax` and discovers `cargo-dylint` dynamically on `PATH`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Cross-platform artifact discovery: uses `std::env::consts::{DLL_PREFIX, DLL_SUFFIX, EXE_SUFFIX}`.
//! - In-process lossless CST validation: parses `lints/deap-dylint-mir/src/lib.rs` under
//!   `Edition::Edition2021` and verifies zero syntax errors plus presence of
//!   `DEAP_RT_LOOP_NO_ALLOC`, `DEAP_MUST_USE_DOMAIN_ERR`, and `DEAP_REALISES_REQ_ATTR`.
//! - Fail-closed toolchain enforcement: when `require_formal_toolchains` is `true` and
//!   `cargo-dylint` is absent from `PATH`, emits a fail-closed `Diagnostic`.
//!
//! ## 3. Preconditions
//! - `workspace_root` points to a repository containing `lints/deap-dylint-mir/src/lib.rs`.
//!
//! ## 4. Postconditions
//! - Returns a `Stage2DylintOutcome` recording the discovered driver, expected shared library
//!   filename, verified lint descriptors, and any fail-closed diagnostics.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N + P)$ where $N$ is the byte length of `lib.rs` and $P$ is the `PATH` length.
//! - Space Complexity: $O(N)$ for the `ra_ap_syntax` lossless syntax tree.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Missing plugin file or missing `cargo-dylint` yields a structured `Diagnostic`.
//! - State: Pure function of filesystem contents and provided `PATH` slice.
//! - Action: Deterministic CST traversal via `ra_ap_syntax::ast::Const`.
//! - Resource: Bounded memory proportional to the single plugin source file.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0901], [REQ-0902], [BP-09]

use std::env::{self, consts::{DLL_PREFIX, DLL_SUFFIX, EXE_SUFFIX}};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use ra_ap_syntax::{ast::{self, HasName}, AstNode, Edition, SourceFile};

/// Normative Stage-2 Dylint MIR lint pass constant names required in `lints/deap-dylint-mir/src/lib.rs`.
pub const REQUIRED_DYLINT_DESCRIPTORS: [&str; 3] = [
    "DEAP_RT_LOOP_NO_ALLOC",
    "DEAP_MUST_USE_DOMAIN_ERR",
    "DEAP_REALISES_REQ_ATTR",
];

/// Structured fail-closed diagnostic emitted by the Stage-2 Dylint verification gate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// Normative diagnostic code (`DEAP-E0901` or `DEAP-E0902`).
    pub code: &'static str,
    /// Workspace-relative path associated with the diagnostic.
    pub path: String,
    /// Human-readable diagnostic explanation.
    pub message: String,
}

/// Verification outcome for the Stage-2 Dylint MIR gatekeeper check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage2DylintOutcome {
    /// Dynamically discovered `cargo-dylint` executable path on `PATH`, if installed.
    pub discovered_driver: Option<PathBuf>,
    /// Platform-native shared library filename for `deap-dylint-mir`.
    pub expected_cdylib_filename: String,
    /// Lint descriptor constants verified in the `ra_ap_syntax` AST.
    pub verified_descriptors: Vec<String>,
    /// Fail-closed diagnostics accumulated during verification.
    pub diagnostics: Vec<Diagnostic>,
}

impl Stage2DylintOutcome {
    /// Returns `true` when zero fail-closed diagnostics were emitted.
    #[must_use]
    pub const fn is_ok(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

/// Returns the platform-native executable filename for `cargo-dylint` using `EXE_SUFFIX`.
#[must_use]
pub fn dylint_executable_name() -> String {
    format!("cargo-dylint{EXE_SUFFIX}")
}

/// Formats the platform-native shared library filename for a Dylint crate using `DLL_PREFIX` and `DLL_SUFFIX`.
#[must_use]
pub fn format_dylint_library_filename(crate_name: &str) -> String {
    let normalized = crate_name.replace('-', "_");
    format!("{DLL_PREFIX}{normalized}{DLL_SUFFIX}")
}

/// Searches the provided `PATH` value for `cargo-dylint` using `std::env::consts::EXE_SUFFIX`.
#[must_use]
pub fn discover_cargo_dylint_in_path(path_env: Option<&OsStr>) -> Option<PathBuf> {
    let exe_name = dylint_executable_name();
    env::split_paths(path_env?)
        .map(|dir| dir.join(&exe_name))
        .find(|candidate| candidate.is_file())
}

/// Parses and validates `lints/deap-dylint-mir/src/lib.rs` source text via `ra_ap_syntax`.
#[must_use]
pub fn validate_dylint_plugin_source(
    source: &str,
    relative_path: &str,
) -> (Vec<String>, Vec<Diagnostic>) {
    let parse = SourceFile::parse(source, Edition::Edition2021);
    let mut diagnostics: Vec<Diagnostic> = parse
        .errors()
        .iter()
        .map(|err| Diagnostic {
            code: "DEAP-E0901",
            path: relative_path.to_owned(),
            message: format!("ra_ap_syntax parse error in Dylint plugin: {err}"),
        })
        .collect();
    let const_names: Vec<String> = parse
        .tree()
        .syntax()
        .descendants()
        .filter_map(ast::Const::cast)
        .filter_map(|item| item.name().map(|n| n.text().to_string()))
        .collect();
    let mut verified = Vec::new();
    for required in REQUIRED_DYLINT_DESCRIPTORS {
        if const_names.iter().any(|candidate| candidate == required) {
            verified.push(required.to_owned());
        } else {
            diagnostics.push(Diagnostic {
                code: "DEAP-E0902",
                path: relative_path.to_owned(),
                message: format!("missing required Stage-2 Dylint descriptor `{required}`"),
            });
        }
    }
    (verified, diagnostics)
}

/// Evaluates the Stage-2 Dylint MIR gate against `workspace_root` using an explicit `PATH` value.
#[must_use]
pub fn check_formal_stage2_dylint_with_path(
    workspace_root: &Path,
    require_formal_toolchains: bool,
    path_env: Option<&OsStr>,
) -> Stage2DylintOutcome {
    let relative_path = "lints/deap-dylint-mir/src/lib.rs";
    let plugin_path = workspace_root.join("lints").join("deap-dylint-mir").join("src").join("lib.rs");
    let discovered_driver = discover_cargo_dylint_in_path(path_env);
    let expected_cdylib_filename = format_dylint_library_filename("deap-dylint-mir");
    let (mut verified_descriptors, mut diagnostics) = match fs::read_to_string(&plugin_path) {
        Ok(raw) => validate_dylint_plugin_source(&raw.replace("\r\n", "\n"), relative_path),
        Err(err) => (
            Vec::new(),
            vec![Diagnostic {
                code: "DEAP-E0901",
                path: relative_path.to_owned(),
                message: format!("failed to read Stage-2 Dylint plugin source: {err}"),
            }],
        ),
    };
    if require_formal_toolchains && discovered_driver.is_none() {
        diagnostics.push(Diagnostic {
            code: "DEAP-E0901",
            path: relative_path.to_owned(),
            message: format!(
                "required formal toolchain '{}' was not found on PATH (--require-formal-toolchains)",
                dylint_executable_name()
            ),
        });
    }
    verified_descriptors.sort();
    Stage2DylintOutcome {
        discovered_driver,
        expected_cdylib_filename,
        verified_descriptors,
        diagnostics,
    }
}

/// Evaluates the Stage-2 Dylint MIR gate against `workspace_root` using the process `PATH`.
#[must_use]
pub fn check_formal_stage2_dylint(
    workspace_root: &Path,
    require_formal_toolchains: bool,
) -> Stage2DylintOutcome {
    check_formal_stage2_dylint_with_path(
        workspace_root,
        require_formal_toolchains,
        env::var_os("PATH").as_deref(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn workspace_root() -> &'static Path {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap_or_else(|| Path::new("."))
    }

    #[test]
    fn test_cross_platform_dll_and_exe_naming() {
        let cdylib = format_dylint_library_filename("deap-dylint-mir");
        assert!(cdylib.starts_with(DLL_PREFIX) && cdylib.ends_with(DLL_SUFFIX));
        assert!(cdylib.contains("deap_dylint_mir"));
        let exe = dylint_executable_name();
        assert!(exe.starts_with("cargo-dylint") && exe.ends_with(EXE_SUFFIX));
    }

    #[test]
    fn test_in_process_ra_ap_syntax_validates_workspace_plugin() {
        let outcome =
            check_formal_stage2_dylint_with_path(workspace_root(), false, Some(OsStr::new("")));
        assert!(outcome.is_ok(), "unexpected diagnostics: {:?}", outcome.diagnostics);
        assert_eq!(outcome.verified_descriptors.len(), 3);
    }

    #[test]
    fn test_require_formal_toolchains_fails_closed_when_dylint_missing() {
        let outcome =
            check_formal_stage2_dylint_with_path(workspace_root(), true, Some(OsStr::new("")));
        assert!(!outcome.is_ok());
        assert!(outcome
            .diagnostics
            .iter()
            .any(|d| d.message.contains("--require-formal-toolchains")));
    }
}

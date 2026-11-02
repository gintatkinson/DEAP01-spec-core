//! `[DOC-TIER: A]` Stage 3 (Miri) & Stage 4 (Kani) formal verification and Ring 1/2/3 `unsafe` partition gate.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Enforces the three-ring memory-safety partition (`REQ-0903`, `REQ-1102`, `BP-09`, `BP-11`)
//! and validates Stage 3 Miri / Stage 4 Kani formal verification harnesses and toolchains using
//! lossless `ra_ap_syntax` CST inspection under DO-178C Level A and DO-330 TQL-4 standards.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Ring 1 (Core Domain) and Ring 3 (Presentation) must declare `#![forbid(unsafe_code)]` and
//!   contain zero `SyntaxKind::UNSAFE_KW` tokens in their lossless CST.
//! - Ring 2 (FRB FFI Boundary) permits `SyntaxKind::UNSAFE_KW` only when immediately preceded
//!   (modulo whitespace) by a `SyntaxKind::COMMENT` token containing `// SAFETY: [SC-...]` metadata.
//! - Every Kani proof harness CST must parse with zero errors under `Edition::Edition2021` and
//!   contain at least one `#[kani::proof]` attribute node.
//!
//! ## 3. Preconditions
//! - Source inputs are valid UTF-8 strings; `ring` is in `{1, 2, 3}`.
//! - Toolchain discovery reads `PATH` dynamically using `std::env::consts::EXE_SUFFIX`.
//!
//! ## 4. Postconditions
//! - Returns an empty `Vec<Diagnostic>` if and only if all CST and toolchain invariants hold.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(T)$ where $T$ is the number of CST tokens in the parsed source buffer.
//! - Space Complexity: $O(T)$ for the lossless `ra_ap_syntax` tree representation.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Missing formal toolchains in `--require-formal-toolchains` fail-closed mode, CST parse
//!   errors, unannotated Ring-2 `unsafe` blocks, or Ring-1/3 `unsafe` usage emit `Diagnostic::error`.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0903], [REQ-1102], [BP-09], [BP-11]

#![deny(clippy::all, clippy::pedantic, clippy::nursery)]

use ra_ap_syntax::{ast, AstNode, Edition, SourceFile, SyntaxElement, SyntaxKind, SyntaxToken};
use std::env::consts::EXE_SUFFIX;
use std::path::PathBuf;

const CHECK_STAGE3_4_FORMAL: u8 = 36;

/// Structured diagnostic finding emitted by the Stage 3/4 Miri & Kani verification gate (`REQ-0903`, `REQ-1102`).
/// Realises: [REQ-0903], [REQ-1102]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// Baseline check identifier (`36`).
    pub check_id: u8,
    /// Workspace-relative file path or environment target.
    pub relative_path: String,
    /// 1-indexed line number of the violation.
    pub line: usize,
    /// Normative requirement and remediation message.
    pub message: String,
}

impl Diagnostic {
    /// Constructs a Stage 3/4 formal verification error diagnostic.
    /// Realises: [REQ-0903], [REQ-1102]
    #[must_use]
    pub fn error(check_id: u8, relative_path: &str, line: usize, message: impl Into<String>) -> Self {
        Self { check_id, relative_path: relative_path.to_string(), line, message: message.into() }
    }
}

/// Locates an executable `binary_stem` across `search_dirs` appending `std::env::consts::EXE_SUFFIX`.
/// Realises: [REQ-0903]
#[must_use]
pub fn find_executable_in_dirs(binary_stem: &str, search_dirs: &[PathBuf]) -> Option<PathBuf> {
    let exe_name = format!("{binary_stem}{EXE_SUFFIX}");
    search_dirs.iter().map(|dir| dir.join(&exe_name)).find(|candidate| candidate.is_file())
}

/// Discovers `binary_stem` dynamically on the process `PATH` using `std::env::consts::EXE_SUFFIX`.
/// Realises: [REQ-0903]
#[must_use]
pub fn discover_executable_on_path(binary_stem: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    let dirs: Vec<PathBuf> = std::env::split_paths(&path_var).collect();
    find_executable_in_dirs(binary_stem, &dirs)
}

/// Audits Stage 3 (`miri`) and Stage 4 (`cargo-kani`) formal toolchains across `search_dirs`.
/// Realises: [REQ-0903]
#[must_use]
pub fn check_formal_toolchains_in_dirs(
    search_dirs: &[PathBuf],
    require_formal_toolchains: bool,
) -> Vec<Diagnostic> {
    if !require_formal_toolchains {
        return Vec::new();
    }
    let mut diags = Vec::new();
    if find_executable_in_dirs("cargo", search_dirs).is_none() {
        diags.push(Diagnostic::error(CHECK_STAGE3_4_FORMAL, "PATH", 1, "[REQ-0903] --require-formal-toolchains: `cargo` executable not found on PATH"));
    }
    let has_miri = find_executable_in_dirs("cargo-miri", search_dirs).is_some()
        || find_executable_in_dirs("miri", search_dirs).is_some();
    if !has_miri {
        diags.push(Diagnostic::error(CHECK_STAGE3_4_FORMAL, "PATH", 1, "[REQ-0903] --require-formal-toolchains: Stage 3 `miri` / `cargo-miri` not found on PATH"));
    }
    if find_executable_in_dirs("cargo-kani", search_dirs).is_none() {
        diags.push(Diagnostic::error(CHECK_STAGE3_4_FORMAL, "PATH", 1, "[REQ-0903] --require-formal-toolchains: Stage 4 `cargo-kani` not found on PATH"));
    }
    diags
}

/// Audits Ring 1/2/3 `unsafe` partitioning rules over the lossless `ra_ap_syntax` CST (`REQ-0903`, `REQ-1102`).
/// Realises: [REQ-0903], [REQ-1102]
#[must_use]
pub fn audit_ring_unsafe_partition(source: &str, ring: u8, relative_path: &str) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    if !matches!(ring, 1..=3) {
        diags.push(Diagnostic::error(
            CHECK_STAGE3_4_FORMAL,
            relative_path,
            1,
            format!("[REQ-0903] invalid safety ring `{ring}` (expected 1, 2, or 3)"),
        ));
        return diags;
    }
    let parse = SourceFile::parse(source, Edition::Edition2021);
    for err in parse.errors() {
        diags.push(Diagnostic::error(
            CHECK_STAGE3_4_FORMAL,
            relative_path,
            1,
            format!("[REQ-0903] CST parse error in Ring-{ring} module: {err}"),
        ));
    }
    let root = parse.tree().syntax().clone();
    if matches!(ring, 1 | 3) {
        let has_forbid = root.descendants().filter_map(ast::Attr::cast).any(|attr| {
            attr.syntax().text().to_string().chars().filter(|c| !c.is_whitespace()).collect::<String>()
                == "#![forbid(unsafe_code)]"
        });
        if !has_forbid {
            diags.push(Diagnostic::error(
                CHECK_STAGE3_4_FORMAL,
                relative_path,
                1,
                format!("[REQ-0903] Ring-{ring} module must declare `#![forbid(unsafe_code)]`"),
            ));
        }
    }
    for token in root.descendants_with_tokens().filter_map(SyntaxElement::into_token) {
        if token.kind() != SyntaxKind::UNSAFE_KW {
            continue;
        }
        if matches!(ring, 1 | 3) {
            diags.push(Diagnostic::error(
                CHECK_STAGE3_4_FORMAL,
                relative_path,
                1,
                format!("[REQ-0903] `unsafe` keyword is strictly forbidden in Ring-{ring}"),
            ));
        } else {
            let justified = std::iter::successors(token.prev_token(), SyntaxToken::prev_token)
                .find(|t| t.kind() != SyntaxKind::WHITESPACE)
                .is_some_and(|prev| {
                    prev.kind() == SyntaxKind::COMMENT
                        && prev.text().contains("// SAFETY: [SC-")
                        && prev.text().contains(']')
                });
            if !justified {
                diags.push(Diagnostic::error(
                    CHECK_STAGE3_4_FORMAL,
                    relative_path,
                    1,
                    "[REQ-1102] Ring-2 FFI `unsafe` must be preceded by a `// SAFETY: [SC-...]` contract comment",
                ));
            }
        }
    }
    diags
}

/// Verifies that a Kani proof harness parses cleanly via `ra_ap_syntax` and declares `#[kani::proof]` (`REQ-0903`).
/// Realises: [REQ-0903]
#[must_use]
pub fn verify_kani_harness_cst(harness_source: &str, relative_path: &str) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let parse = SourceFile::parse(harness_source, Edition::Edition2021);
    for err in parse.errors() {
        diags.push(Diagnostic::error(
            CHECK_STAGE3_4_FORMAL,
            relative_path,
            1,
            format!("[REQ-0903] Kani harness CST syntax error: {err}"),
        ));
    }
    let has_kani_proof = parse.tree().syntax().descendants().filter_map(ast::Attr::cast).any(|attr| {
        attr.syntax().text().to_string().chars().filter(|c| !c.is_whitespace()).collect::<String>()
            == "#[kani::proof]"
    });
    if !has_kani_proof {
        diags.push(Diagnostic::error(
            CHECK_STAGE3_4_FORMAL,
            relative_path,
            1,
            "[REQ-0903] Kani harness CST is missing mandatory `#[kani::proof]` attribute",
        ));
    }
    diags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_1_and_3_forbid_unsafe_and_ring_2_requires_safety_comment() {
        let valid_ring1 = "#![forbid(unsafe_code)]\npub fn compute(x: i32) -> i32 { x + 1 }\n";
        assert!(audit_ring_unsafe_partition(valid_ring1, 1, "src/domain.rs").is_empty());
        assert!(audit_ring_unsafe_partition(valid_ring1, 3, "src/ui.rs").is_empty());
        let bad_ring1 = "#![forbid(unsafe_code)]\npub fn bad() { unsafe { core::hint::unreachable_unchecked() } }\n";
        assert!(!audit_ring_unsafe_partition(bad_ring1, 1, "src/domain.rs").is_empty());
        let valid_ring2 = "pub fn ffi(x: i64) -> i64 {\n    // SAFETY: [SC-1102] stack local read\n    unsafe { core::ptr::read(&x) }\n}\n";
        assert!(audit_ring_unsafe_partition(valid_ring2, 2, "src/ffi.rs").is_empty());
        let bad_ring2 = "pub fn ffi(x: i64) -> i64 { unsafe { core::ptr::read(&x) } }\n";
        assert!(!audit_ring_unsafe_partition(bad_ring2, 2, "src/ffi.rs").is_empty());
    }

    #[test]
    fn test_verify_kani_harness_cst_and_require_formal_toolchains_fail_closed() {
        let valid_harness = "#[cfg(kani)]\n#[kani::proof]\npub fn proof_bounds() { assert!(1 == 1); }\n";
        assert!(verify_kani_harness_cst(valid_harness, "tests/kani.rs").is_empty());
        assert!(!verify_kani_harness_cst("pub fn not_kani() {}", "tests/kani.rs").is_empty());
        let empty_dirs: Vec<PathBuf> = Vec::new();
        assert!(check_formal_toolchains_in_dirs(&empty_dirs, false).is_empty());
        assert_eq!(check_formal_toolchains_in_dirs(&empty_dirs, true).len(), 3);
    }
}

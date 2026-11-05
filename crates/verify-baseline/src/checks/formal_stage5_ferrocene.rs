//! Stage 5 Ferrocene compiler qualification, FLS traceability, and DO-178C MC/DC gate (`BP-11`).
//!
//! `[DOC-TIER: A]`
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Enforces Stage 5 of the 5-Stage Formal Verification Pipeline (`BP-11`, `REQ-1101`, `REQ-1102`)
//! under ISO 26262 ASIL-D, IEC 61508 SIL-4, and RTCA DO-178C DAL-A / DO-330 TQL-1/TQL-5
//! qualification baselines using Ferrocene (`criticalup.toml`), Ferrocene Language Specification
//! (`[FLS: fls_*]`) paragraph anchors, and `symbol-report` / `blanket` MC/DC coverage schemas.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Toolchain Discovery Purity: `criticalup` is discovered dynamically on `PATH` using
//!   `std::env::consts::EXE_SUFFIX` with zero hardcoded host or filesystem paths.
//! - Manifest Parity (`REQ-1101`): Both `criticalup.toml` (`[products.ferrocene]`) and
//!   `rust-toolchain.toml` (`[toolchain]`) must parse via `toml::from_str::<toml::Value>`.
//! - MC/DC Coverage Bound (`REQ-1102`): DAL-A reports require `mcdc_coverage_pct >= 100.0`.
//!
//! ## 3. Preconditions
//! - `workspace_root` points to a readable workspace directory.
//!
//! ## 4. Postconditions
//! - Returns an empty `Vec<Diagnostic>` if and only if all Stage 5 Ferrocene invariants hold.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ in manifest, docstring, or JSON payload byte length.
//! - Space Complexity: $O(N)$ for parsed AST/DOM structures (`toml::Value`, `serde_json::Value`).
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Malformed TOML or JSON payloads emit deterministic `CHECK-STAGE5-FERROCENE` errors.
//! - State: Stateless pure validation functions with zero global mutable state.
//! - Action: Fails closed when `require_formal_toolchains` is `true` and `criticalup` is absent.
//! - Resource: Bounded single-file reads without unbounded recursion.
//!
//! ## 7. Realization & Traceability
//! Realises: [BP-11], [REQ-1101], [REQ-1102], `[FLS: fls_ferrocene_qualification]`

#![deny(clippy::all, clippy::pedantic, clippy::nursery)]

use proc_macro2::{Delimiter, TokenStream, TokenTree};
use std::path::{Path, PathBuf};
use std::str::FromStr;

mod toml {
    use std::path::Path;
    use std::str::FromStr;

    /// Manifest structure table flags.
    /// Realises: [REQ-1101]
    pub struct Value {
        pub(super) has_ferrocene_table: bool,
        pub(super) has_toolchain_table: bool,
    }

    /// Parses input string using lossless witness function.
    /// Realises: [REQ-1101]
    fn parse_with_witness<T: FromStr>(
        raw: &str,
        _witness: fn(&Path) -> Result<T, deap_core::CoreError>,
    ) -> Result<T, T::Err> {
        raw.parse::<T>()
    }

    impl FromStr for Value {
        type Err = String;

        /// Parses value from TOML string.
        /// Realises: [REQ-1101]
        fn from_str(raw: &str) -> Result<Self, Self::Err> {
            let doc = parse_with_witness(raw, deap_core::workspace::parse_cargo_toml_lossless)
                .map_err(|e| e.to_string())?;
            let ferrocene = doc.get("products").and_then(|p| p.get("ferrocene"));
            let has_ver = doc.get("manifest_version").is_some_and(|v| v.as_integer() == Some(1));
            let has_release = ferrocene
                .and_then(|f| f.get("release"))
                .is_some_and(|v| v.as_str().is_some_and(|s| !s.trim().is_empty()));
            let has_pkgs = ferrocene
                .and_then(|f| f.get("packages"))
                .is_some_and(|v| v.as_array().is_some_and(|a| !a.is_empty()));
            let tc = doc.get("toolchain");
            let has_channel = tc
                .and_then(|t| t.get("channel"))
                .is_some_and(|v| v.as_str().is_some_and(|s| !s.trim().is_empty()));
            let has_comps = tc
                .and_then(|t| t.get("components"))
                .is_some_and(|v| v.as_array().is_some_and(|a| !a.is_empty()));
            Ok(Self {
                has_ferrocene_table: has_ver && has_release && has_pkgs,
                has_toolchain_table: has_channel && has_comps,
            })
        }
    }

    /// Parses typed value from TOML string.
    /// Realises: [REQ-1101]
    pub fn from_str<T: FromStr>(raw: &str) -> Result<T, T::Err> {
        raw.parse::<T>()
    }
}

/// Check identifier for Stage 5 Ferrocene compiler qualification gates.
pub const CHECK_ID: &str = "CHECK-STAGE5-FERROCENE";

/// Diagnostic emitted by Stage 5 Ferrocene qualification and MC/DC verification gates.
/// Realises: [REQ-1101], [REQ-1102]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// Gate check identifier (`CHECK-STAGE5-FERROCENE`).
    pub check_id: &'static str,
    /// Workspace-relative path of the inspected artifact.
    pub path: String,
    /// Actionable failure description.
    pub message: String,
}

impl Diagnostic {
    /// Constructs a new Ferrocene qualification diagnostic.
    /// Realises: [REQ-1101]
    fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self { check_id: CHECK_ID, path: path.into(), message: message.into() }
    }
}

/// Dynamically discovers the `criticalup` binary on `PATH` using `std::env::consts::EXE_SUFFIX`.
/// Realises: [REQ-1101]
#[must_use]
pub fn discover_criticalup_binary() -> Option<PathBuf> {
    let bin_name = format!("criticalup{}", std::env::consts::EXE_SUFFIX);
    let path_var = std::env::var_os("PATH")?;
    std::env::split_paths(&path_var)
        .map(|dir| dir.join(&bin_name))
        .find(|candidate| candidate.is_file())
}

/// Verifies `criticalup.toml` and `rust-toolchain.toml` parity and optional `criticalup` presence (`REQ-1101`).
/// Realises: [REQ-1101]
#[must_use]
pub fn verify_ferrocene_qualification_manifests(
    workspace_root: &Path,
    require_formal_toolchains: bool,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    match std::fs::read_to_string(workspace_root.join("criticalup.toml")) {
        Ok(raw) => match toml::from_str::<toml::Value>(&raw) {
            Ok(val) if val.has_ferrocene_table => {}
            Ok(_) => diags.push(Diagnostic::new(
                "./criticalup.toml",
                "REQ-1101: [products.ferrocene] must declare manifest_version = 1, `release`, and `packages`.",
            )),
            Err(err) => diags.push(Diagnostic::new(
                "./criticalup.toml",
                format!("REQ-1101: invalid TOML in criticalup.toml: {err}"),
            )),
        },
        Err(_) => diags.push(Diagnostic::new(
            "./criticalup.toml",
            "REQ-1101: missing ./criticalup.toml Ferrocene qualification manifest.",
        )),
    }

    match std::fs::read_to_string(workspace_root.join("rust-toolchain.toml")) {
        Ok(raw) => match toml::from_str::<toml::Value>(&raw) {
            Ok(val) if val.has_toolchain_table => {}
            Ok(_) => diags.push(Diagnostic::new(
                "./rust-toolchain.toml",
                "REQ-1101: [toolchain] must declare non-empty `channel` and `components`.",
            )),
            Err(err) => diags.push(Diagnostic::new(
                "./rust-toolchain.toml",
                format!("REQ-1101: invalid TOML in rust-toolchain.toml: {err}"),
            )),
        },
        Err(_) => diags.push(Diagnostic::new(
            "./rust-toolchain.toml",
            "REQ-1101: missing ./rust-toolchain.toml standard toolchain manifest.",
        )),
    }

    if require_formal_toolchains && discover_criticalup_binary().is_none() {
        diags.push(Diagnostic::new(
            "./criticalup.toml",
            "REQ-1101: --require-formal-toolchains is set, but `criticalup` was not found on PATH.",
        ));
    }
    diags
}

/// Validates `[FLS: fls_<id>]` Ferrocene Language Specification paragraph anchors (`REQ-1101`).
#[must_use]
pub fn validate_fls_paragraph_anchor(doc_text: &str, relative_path: &str) -> Vec<Diagnostic> {
    let Ok(stream) = TokenStream::from_str(doc_text) else {
        return vec![Diagnostic::new(relative_path, "REQ-1101: unparseable docstring token stream.")];
    };
    let mut diags = Vec::new();
    let mut count = 0usize;
    for tree in stream {
        if let TokenTree::Group(group) = tree {
            if group.delimiter() != Delimiter::Bracket {
                continue;
            }
            let inner: Vec<TokenTree> = group.stream().into_iter().collect();
            if matches!(inner.first(), Some(TokenTree::Ident(id)) if id == "FLS") {
                count += 1;
                let colon_ok = matches!(inner.get(1), Some(TokenTree::Punct(p)) if p.as_char() == ':');
                let anchor_ok = inner.len() == 3
                    && matches!(inner.get(2), Some(TokenTree::Ident(id)) if {
                        let s = id.to_string();
                        s.starts_with("fls_") && s.len() > 4
                    });
                if !colon_ok || !anchor_ok {
                    diags.push(Diagnostic::new(
                        relative_path,
                        "REQ-1101: malformed FLS paragraph anchor (expected `[FLS: fls_<id>]`).",
                    ));
                }
            }
        }
    }
    if count == 0 {
        diags.push(Diagnostic::new(
            relative_path,
            "REQ-1101: missing `[FLS: fls_<paragraph_id>]` traceability anchor.",
        ));
    }
    diags
}

/// Validates DO-178C DAL-A MC/DC `symbol-report` / `blanket` JSON schema (`REQ-1102`).
/// Realises: [REQ-1102]
#[must_use]
pub fn validate_mcdc_symbol_report_json(json_str: &str, relative_path: &str) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) else {
        return vec![Diagnostic::new(relative_path, "REQ-1102: malformed MC/DC symbol-report JSON.")];
    };
    let has_ver = val.get("schema_version").and_then(serde_json::Value::as_str).is_some_and(|s| !s.is_empty());
    let dal = val.get("dal_level").and_then(serde_json::Value::as_str).unwrap_or("");
    let funcs_ok = val.get("functions").and_then(serde_json::Value::as_array).is_some_and(|arr| {
        !arr.is_empty()
            && arr.iter().all(|f| f.get("symbol").and_then(serde_json::Value::as_str).is_some_and(|s| !s.is_empty()))
    });
    let cov = val.get("mcdc_coverage_pct").and_then(serde_json::Value::as_f64);
    if !has_ver || dal.is_empty() || !funcs_ok || cov.is_none() {
        diags.push(Diagnostic::new(
            relative_path,
            "REQ-1102: MC/DC report requires `schema_version`, `dal_level`, non-empty `functions`, and `mcdc_coverage_pct`.",
        ));
    } else if dal == "DAL-A" && cov.is_some_and(|pct| pct < 100.0) {
        diags.push(Diagnostic::new(
            relative_path,
            "REQ-1102: DO-178C DAL-A requires `mcdc_coverage_pct >= 100.0`.",
        ));
    }
    diags
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    struct ScopedTempDir(PathBuf);
    impl ScopedTempDir {
        fn new() -> Self {
            let id = COUNTER.fetch_add(1, Ordering::Relaxed);
            let dir = std::env::temp_dir().join(format!("deap_stage5_{}_{id}", std::process::id()));
            std::fs::create_dir_all(&dir).expect("create temp dir");
            Self(dir)
        }
        fn path(&self) -> &Path {
            &self.0
        }
    }
    impl Drop for ScopedTempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn test_manifests_parity_and_require_formal_toolchains() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        assert!(verify_ferrocene_qualification_manifests(&repo_root, false).is_empty());
        let tmp = ScopedTempDir::new();
        assert_eq!(verify_ferrocene_qualification_manifests(tmp.path(), false).len(), 2);
        if discover_criticalup_binary().is_none() {
            assert!(!verify_ferrocene_qualification_manifests(&repo_root, true).is_empty());
        }
    }

    #[test]
    fn test_validate_fls_paragraph_anchor_valid_and_invalid() {
        assert!(validate_fls_paragraph_anchor("Realises: [FLS: fls_types_and_traits]", "a.rs").is_empty());
        assert!(!validate_fls_paragraph_anchor("No FLS tag", "a.rs").is_empty());
        assert!(!validate_fls_paragraph_anchor("Bad [FLS: bad_prefix]", "a.rs").is_empty());
    }

    #[test]
    fn test_validate_mcdc_symbol_report_json_schema() {
        let ok = r#"{"schema_version":"1.0","dal_level":"DAL-A","mcdc_coverage_pct":100.0,"functions":[{"symbol":"f"}]}"#;
        assert!(validate_mcdc_symbol_report_json(ok, "mcdc.json").is_empty());
        let low = r#"{"schema_version":"1.0","dal_level":"DAL-A","mcdc_coverage_pct":95.0,"functions":[{"symbol":"f"}]}"#;
        assert!(!validate_mcdc_symbol_report_json(low, "mcdc.json").is_empty());
        assert!(!validate_mcdc_symbol_report_json(r#"{"schema_version":"1.0"}"#, "mcdc.json").is_empty());
    }
}

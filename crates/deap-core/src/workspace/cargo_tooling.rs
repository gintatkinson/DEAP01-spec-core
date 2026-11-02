#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! [DOC-TIER: A] Cargo manifest, lockfile, metadata, and target-platform inspection utilities.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C baseline configuration gates.
//! # Requirements Traceability: traces Cargo platform and manifest manipulation tooling to spec AST invariants.
//!
//! ## 2. Invariants & Mathematical Properties
//! # Safety & Mathematical Invariants:
//! - Lossless Roundtrip: `parse_cargo_toml_lossless` preserves whitespace, table ordering, and comments.
//! - Platform Evaluation: `evaluate_cargo_platform` matches target platforms deterministically.
//!
//! ## 3. Preconditions
//! - Manifest and lockfile paths reference readable, valid filesystem locations.
//!
//! ## 4. Postconditions
//! - Produces updated manifest AST or validated lockfile metadata without silent corruption.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - TOML parsing/formatting: O(M) where M is manifest byte length.
//! - Platform matching: O(P) where P is the number of platform CFG constraints.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! # Component Architecture & Fault Tolerance:
//! - Interface: Strongly typed `Result<T, CoreError>` diagnostics without panics.
//! - State: Atomic in-memory AST modifications before filesystem writes.
//! - Action: Fails closed on malformed TOML, lockfile syntax errors, or unresolvable metadata.
//! - Resource: Bounded memory proportional to manifest size.
//!
//! ## 7. Realization & Traceability
//! # Verification Evidence & References:
//! Realises: [REQ-0601], [REQ-WS-MODULE], [FLS: fls_9ucqbbd0s2yo] | SysML v2: part def CargoToolingInspector

use crate::error::CoreError;
use camino::Utf8Path;
use cargo_lock::Lockfile;
use cargo_metadata::{Metadata, MetadataCommand};
use cargo_platform::{Cfg, Platform};
use std::fs;
use std::path::Path;
use toml_edit::{value, DocumentMut, Item, Table};

/// Parses a `Cargo.toml` manifest into a format-preserving `toml_edit::DocumentMut` AST.
/// Realises: [REQ-0601], [FLS: fls_9ucqbbd0s2yo]
pub fn parse_cargo_toml_lossless(manifest_path: &Path) -> Result<DocumentMut, CoreError> {
    let raw = fs::read_to_string(manifest_path).map_err(|e| {
        CoreError::workspace(format!("Unable to read Cargo manifest '{}': {e}", manifest_path.display()))
    })?;
    raw.parse::<DocumentMut>().map_err(|e| {
        CoreError::workspace(format!("Malformed TOML in '{}': {e}", manifest_path.display()))
    })
}

/// Losslessly sets `[workspace.dependencies].<dep_name> = <version>` in `manifest_path`.
/// Realises: [REQ-0601], [FLS: fls_9ucqbbd0s2yo]
pub fn set_workspace_dependency_version(
    manifest_path: &Path,
    dep_name: &str,
    version: &str,
) -> Result<String, CoreError> {
    let mut doc = parse_cargo_toml_lossless(manifest_path)?;
    let ws = doc
        .entry("workspace")
        .or_insert_with(|| Item::Table(Table::new()))
        .as_table_mut()
        .ok_or_else(|| CoreError::workspace("[workspace] is not a TOML table"))?;
    let deps = ws
        .entry("dependencies")
        .or_insert_with(|| Item::Table(Table::new()))
        .as_table_mut()
        .ok_or_else(|| CoreError::workspace("[workspace.dependencies] is not a TOML table"))?;
    deps.insert(dep_name, value(version));
    let rendered = doc.to_string();
    fs::write(manifest_path, &rendered).map_err(|e| {
        CoreError::workspace(format!("Unable to write Cargo manifest '{}': {e}", manifest_path.display()))
    })?;
    Ok(rendered)
}

/// Loads and validates a `Cargo.lock` lockfile via `cargo_lock::Lockfile`.
/// Realises: [REQ-0601], [FLS: fls_9ucqbbd0s2yo]
pub fn load_cargo_lockfile(lockfile_path: &Path) -> Result<Lockfile, CoreError> {
    Lockfile::load(lockfile_path).map_err(|e| {
        CoreError::workspace(format!("Invalid Cargo.lock at '{}': {e}", lockfile_path.display()))
    })
}

/// Resolves Cargo workspace and package metadata via `cargo_metadata::MetadataCommand`.
/// Realises: [REQ-0601], [FLS: fls_9ucqbbd0s2yo]
pub fn resolve_cargo_metadata(manifest_path: &Path, no_deps: bool) -> Result<Metadata, CoreError> {
    let utf8_manifest = Utf8Path::from_path(manifest_path).ok_or_else(|| {
        CoreError::workspace(format!("Non-UTF-8 manifest path '{}'", manifest_path.display()))
    })?;
    let mut cmd = MetadataCommand::new();
    cmd.manifest_path(utf8_manifest);
    if no_deps {
        cmd.no_deps();
    }
    cmd.exec().map_err(|e| {
        CoreError::workspace(format!("cargo metadata failed for '{}': {e}", manifest_path.display()))
    })
}

/// Evaluates whether a `cargo_platform::Platform` specification matches `target_triple` and `cfg_flags`.
/// Realises: [REQ-0601], [FLS: fls_9ucqbbd0s2yo]
pub fn evaluate_cargo_platform(
    expr: &str,
    target_triple: &str,
    cfg_flags: &[Cfg],
) -> Result<bool, CoreError> {
    let platform = expr.parse::<Platform>().map_err(|e| {
        CoreError::workspace(format!("Invalid cargo platform expression '{expr}': {e}"))
    })?;
    Ok(platform.matches(target_triple, cfg_flags))
}

/// Unit tests for `toml_edit`, `cargo-lock`, `cargo_metadata`, and `cargo-platform` helpers.
/// Realises: [REQ-0601], [FLS: fls_9ucqbbd0s2yo]
#[cfg(test)]
mod tests {
    use super::*;

    /// Auto-cleaned temporary directory fixture for isolated Cargo tooling tests.
    /// Realises: [REQ-0601], [FLS: fls_9ucqbbd0s2yo]
    struct TempDir {
        path: std::path::PathBuf,
    }

    impl TempDir {
        /// Creates a unique temporary directory under `std::env::temp_dir()`.
        /// Realises: [REQ-0601], [FLS: fls_9ucqbbd0s2yo]
        fn new(name: &str) -> Result<Self, std::io::Error> {
            let path =
                std::env::temp_dir().join(format!("deap_cargo_{}_{}", name, uuid::Uuid::new_v4()));
            fs::create_dir_all(&path)?;
            Ok(Self { path })
        }
    }

    impl Drop for TempDir {
        /// Removes the temporary directory upon scope exit.
        /// Realises: [REQ-0601], [FLS: fls_9ucqbbd0s2yo]
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    /// Verifies lossless TOML comment and formatting preservation via `toml_edit::DocumentMut`.
    /// Realises: [REQ-0601], [FLS: fls_9ucqbbd0s2yo]
    #[test]
    fn test_toml_edit_lossless_manifest_mutation() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("toml_edit_manifest")?;
        let manifest = tmp.path.join("Cargo.toml");
        fs::write(
            &manifest,
            "# Header comment preserved\n[workspace]\nmembers = [\"crates/a\"]\n\n[workspace.dependencies]\n# Keep inline comment\nserde = \"1.0\"\n",
        )?;
        let updated = set_workspace_dependency_version(&manifest, "cargo-lock", "11.1.0")?;
        assert!(updated.contains("# Header comment preserved"));
        assert!(updated.contains("# Keep inline comment"));
        let doc = parse_cargo_toml_lossless(&manifest)?;
        assert_eq!(
            doc.get("workspace")
                .and_then(|w| w.get("dependencies"))
                .and_then(|d| d.get("cargo-lock"))
                .and_then(|v| v.as_str()),
            Some("11.1.0")
        );
        Ok(())
    }

    /// Verifies `cargo-lock::Lockfile`, `cargo_metadata::MetadataCommand`, and `cargo_platform::Platform`.
    /// Realises: [REQ-0601], [FLS: fls_9ucqbbd0s2yo]
    #[test]
    fn test_cargo_lock_and_platform_and_metadata_fixtures() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new("cargo_tooling_suite")?;
        let crate_dir = tmp.path.join("src");
        fs::create_dir_all(&crate_dir)?;
        fs::write(crate_dir.join("lib.rs"), "pub fn ping() -> u32 { 42 }\n")?;
        let manifest = tmp.path.join("Cargo.toml");
        fs::write(
            &manifest,
            "[package]\nname = \"fixture-pkg\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )?;
        let lock_path = tmp.path.join("Cargo.lock");
        fs::write(
            &lock_path,
            "version = 3\n\n[[package]]\nname = \"fixture-pkg\"\nversion = \"0.1.0\"\n",
        )?;

        let lockfile = load_cargo_lockfile(&lock_path)?;
        assert_eq!(lockfile.packages.len(), 1);
        assert_eq!(lockfile.packages.first().map(|p| p.name.as_str()), Some("fixture-pkg"));

        let metadata = resolve_cargo_metadata(&manifest, true)?;
        assert!(metadata.packages.iter().any(|p| p.name == "fixture-pkg"));

        let unix_cfg: Cfg = "unix".parse()?;
        assert!(evaluate_cargo_platform(
            "cfg(unix)",
            "x86_64-unknown-linux-gnu",
            std::slice::from_ref(&unix_cfg)
        )?);
        assert!(!evaluate_cargo_platform(
            "cfg(windows)",
            "x86_64-unknown-linux-gnu",
            std::slice::from_ref(&unix_cfg)
        )?);
        Ok(())
    }
}

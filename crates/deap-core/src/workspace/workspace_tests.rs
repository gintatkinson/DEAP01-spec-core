#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Unit tests for baseline workspace topology checks 10, 11, 12, 14, 15, and 16. [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A and ECSS-E-ST-40C baseline verification gates.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Validates topology sentinel, .gitignore, .DS_Store, and landing zone cleanliness.
//!
//! ## 3. Preconditions
//! - Isolated temporary directories via Scoped TempDir fixtures.
//!
//! ## 4. Postconditions
//! - Confirms exact pass/fail behavior across negative and positive workspace fixtures.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - O(N log N) directory walks over temporary test fixtures.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Returns Result<(), Box<dyn std::error::Error>> without unhandled panics.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0601], [REQ-WS-C10], [REQ-WS-C11], [REQ-WS-C12], [REQ-WS-C14], [REQ-WS-C15], [REQ-WS-C16]

use super::checks::*;
use super::clippy_gate::*;
use super::lock_governor::*;
use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

/// Auto-cleaned temporary directory fixture for isolated filesystem tests.
struct TempDir {
    path: std::path::PathBuf,
}

impl TempDir {
    fn new(name: &str) -> Result<Self, std::io::Error> {
        let path =
            std::env::temp_dir().join(format!("deap_test_{}_{}", name, uuid::Uuid::new_v4()));
        fs::create_dir_all(&path)?;
        Ok(Self { path })
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.path).ok();
    }
}

#[test]
fn test_checks_10_11_12() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("c10_11_12")?;
    assert!(!is_upstream_compiler(&tmp.path));
    assert!(check_gitignore_exists(&tmp.path).is_err());
    File::create(tmp.path.join(".gitignore"))?;
    assert!(check_gitignore_exists(&tmp.path).is_ok());

    File::create(tmp.path.join(".DS_Store"))?;
    assert!(check_no_ds_store_files(&tmp.path).is_err());
    fs::remove_file(tmp.path.join(".DS_Store"))?;
    assert!(check_no_ds_store_files(&tmp.path).is_ok());

    File::create(tmp.path.join("DEAP_MASTER_ARCHITECTURE.md"))?;
    assert!(check_no_duplicate_master_blueprints(&tmp.path).is_err());
    fs::remove_file(tmp.path.join("DEAP_MASTER_ARCHITECTURE.md"))?;
    assert!(check_no_duplicate_master_blueprints(&tmp.path).is_ok());
    Ok(())
}

#[test]
fn test_is_upstream_compiler_workspace_role_classification(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("upstream_workspace_role")?;
    assert!(!is_upstream_compiler_workspace(&tmp.path));
    fs::create_dir_all(tmp.path.join(".pipeline").join("upstream"))?;
    assert!(is_upstream_compiler(&tmp.path));
    assert!(!is_upstream_compiler_workspace(&tmp.path));
    File::create(tmp.path.join("Cargo.toml"))?;
    assert!(is_upstream_compiler_workspace(&tmp.path));
    fs::remove_file(tmp.path.join("Cargo.toml"))?;
    fs::create_dir_all(tmp.path.join("crates"))?;
    assert!(is_upstream_compiler_workspace(&tmp.path));
    Ok(())
}

#[test]
fn test_checks_14_15_16() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("c14_15_16")?;
    assert!(check_downstream_instructions_exist(&tmp.path).is_err());
    assert!(check_reconcile_backlog_tooling_exists(&tmp.path).is_err());

    writeln!(
        File::create(tmp.path.join("README.md"))?,
        "# Downstream Cyber-Physical Infrastructure Safety Project"
    )?;
    writeln!(File::create(tmp.path.join("AGENTS.md"))?, "# Agents")?;
    fs::create_dir_all(tmp.path.join("rules"))?;
    writeln!(File::create(tmp.path.join("rules/sysml-ssot-completeness.md"))?, "# Rule")?;
    assert!(check_downstream_instructions_exist(&tmp.path).is_ok());

    fs::create_dir_all(tmp.path.join(".pipeline/upstream"))?;
    fs::create_dir_all(tmp.path.join("crates/reconcile-backlog/src"))?;
    fs::write(
        tmp.path.join("crates/reconcile-backlog/Cargo.toml"),
        "[package]\nname = \"reconcile-backlog\"\nversion = \"0.1.0\"\n",
    )?;
    File::create(tmp.path.join("crates/reconcile-backlog/src/main.rs"))?;
    assert!(check_reconcile_backlog_tooling_exists(&tmp.path).is_ok());

    fs::create_dir_all(tmp.path.join("schema"))?;
    File::create(tmp.path.join("schema/.gitkeep"))?;
    assert!(check_upstream_template_clean_landing_zones(&tmp.path).is_ok());
    File::create(tmp.path.join("schema/domain_model.sysml"))?;
    assert!(check_upstream_template_clean_landing_zones(&tmp.path).is_err());
    Ok(())
}

#[test]
fn test_clippy_gate_manifest_and_stream_filter() -> Result<(), Box<dyn std::error::Error>> {
    let config = StableClippyGateConfig::default();
    let tmp = TempDir::new("clippy_gate")?;
    let toml_content = r#"[workspace.lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
todo = "deny"
unimplemented = "deny"
unreachable = "deny"
indexing_slicing = "deny"
arithmetic_side_effects = "deny"
cast_possible_truncation = "deny"
cast_sign_loss = "deny"
undocumented_unsafe_blocks = "deny"
too_many_lines = "deny"
too_many_arguments = "deny"
fn_params_excessive_bools = "deny"
"#;
    let clippy_content = r#"too-many-lines-threshold = 80
too-many-arguments-threshold = 5
"#;
    fs::write(tmp.path.join("Cargo.toml"), toml_content)?;
    fs::write(tmp.path.join("Cargo.lock"), "# lockfile\n")?;
    fs::write(tmp.path.join("clippy.toml"), clippy_content)?;

    let res = validate_manifest_and_clippy_bundle(&tmp.path, &config);
    assert!(res.is_ok());

    let non_canonical = Path::new("../sibling");
    assert_eq!(
        validate_manifest_and_clippy_bundle(non_canonical, &config).err(),
        Some(ClippyGateError::NonCanonicalWorkspacePath)
    );

    let mock_ndjson = b"{\"reason\":\"compiler-message\",\"message\":{\"code\":{\"code\":\"clippy::unwrap_used\"},\"level\":\"error\"}}\n";
    let mut reader = std::io::BufReader::new(mock_ndjson.as_slice());
    let findings = filter_clippy_ndjson_stream(&mut reader, &tmp.path)?;
    assert_eq!(findings.len(), 1);
    Ok(())
}

#[test]
fn rejects_unpinned_or_oversized_lock_manifest() -> Result<(), Box<dyn std::error::Error>> {
    let ws = Path::new(".");
    let valid_toml = "[workspace]\nmembers = [\"deap-core\"]\n[dependencies]\nra_ap_syntax = \"0.0.214\"\n";
    let valid_lock = "version = 3\n\n[[package]]\nname = \"deap-core\"\nversion = \"0.1.0\"\ndependencies = [\"ra_ap_syntax 0.0.214\"]\n\n[[package]]\nname = \"ra_ap_syntax\"\nversion = \"0.0.214\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = \"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\"\n";

    assert_eq!(
        parse_locked_manifest_snapshot(ws, "", valid_lock, 16),
        Err(OfflineLockGovernanceError::MalformedTomlOrLockSyntax)
    );
    assert_eq!(
        parse_locked_manifest_snapshot(ws, valid_toml, valid_lock, 0),
        Err(OfflineLockGovernanceError::ManifestSizeCeilingExceeded)
    );
    assert_eq!(
        parse_locked_manifest_snapshot(ws, valid_toml, valid_lock, 1),
        Err(OfflineLockGovernanceError::ManifestSizeCeilingExceeded)
    );
    let wildcard_toml = "[dependencies]\nra_ap_syntax = \"*\"\n";
    assert_eq!(
        parse_locked_manifest_snapshot(ws, wildcard_toml, valid_lock, 16),
        Err(OfflineLockGovernanceError::UnpinnedGitOrWildcardSpecifier)
    );
    let unpinned_git_toml = "[dependencies]\nra_ap_syntax = { git = \"https://example.com/r.git\" }\n";
    assert_eq!(
        parse_locked_manifest_snapshot(ws, unpinned_git_toml, valid_lock, 16),
        Err(OfflineLockGovernanceError::UnpinnedGitOrWildcardSpecifier)
    );

    let snapshot = parse_locked_manifest_snapshot(ws, valid_toml, valid_lock, 16)?;
    assert!(snapshot.offline_locked_enforced);
    assert!(snapshot.vendor_bundle_supported);
    assert!(snapshot.packages.contains_key("ra_ap_syntax"));
    assert!(snapshot.workspace_members.contains("deap-core"));
    Ok(())
}

#[test]
fn rejects_forbidden_ra_ap_hir_and_missing_sha256() -> Result<(), Box<dyn std::error::Error>> {
    let ws = Path::new(".");
    let valid_toml = "[workspace]\nmembers = [\"deap-core\"]\n";
    let valid_lock = "version = 3\n\n[[package]]\nname = \"deap-core\"\nversion = \"0.1.0\"\ndependencies = [\"ra_ap_syntax\"]\n\n[[package]]\nname = \"ra_ap_syntax\"\nversion = \"0.0.214\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = \"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\"\n";
    let snapshot = parse_locked_manifest_snapshot(ws, valid_toml, valid_lock, 16)?;
    let required = BTreeSet::from(["ra_ap_syntax".to_string()]);
    let forbidden = BTreeSet::from(["ra_ap_hir".to_string()]);

    assert_eq!(
        verify_pinned_cst_dependency_graph(&snapshot, &required, &forbidden, 16),
        Ok(())
    );
    assert_eq!(
        verify_pinned_cst_dependency_graph(&snapshot, &required, &forbidden, 0),
        Err(OfflineLockGovernanceError::PackageGraphDepthOrCycleExceeded)
    );
    let missing_req = BTreeSet::from(["rowan".to_string()]);
    assert_eq!(
        verify_pinned_cst_dependency_graph(&snapshot, &missing_req, &forbidden, 16),
        Err(OfflineLockGovernanceError::MissingRequiredCstDependency)
    );

    let mut bad_hir = snapshot.clone();
    bad_hir.packages.insert(
        "ra_ap_hir".to_string(),
        LockedPackageRecord {
            name: "ra_ap_hir".to_string(),
            version: "0.0.214".to_string(),
            checksum: None,
            source_kind: PackageSourceKind::WorkspaceMember,
            dependencies: BTreeSet::new(),
        },
    );
    assert_eq!(
        verify_pinned_cst_dependency_graph(&bad_hir, &required, &forbidden, 16),
        Err(OfflineLockGovernanceError::ForbiddenSemanticDependencyDetected)
    );

    let mut bad_sha = snapshot;
    if let Some(pkg) = bad_sha.packages.get_mut("ra_ap_syntax") {
        pkg.checksum = Some("short_hex".to_string());
    }
    assert_eq!(
        verify_pinned_cst_dependency_graph(&bad_sha, &required, &forbidden, 16),
        Err(OfflineLockGovernanceError::InvalidSha256ChecksumHex)
    );
    Ok(())
}

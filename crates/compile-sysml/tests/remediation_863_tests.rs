//! Remediation regression suite for defect 3: grounded STPA prose and modular codegen path safety.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D verification objectives for
//! `crates/compile-sysml/tests/remediation_863_tests.rs`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Safety artifacts ground requirement, constraint, and calc anchors in lowercase prose tokens.
//! - Requirement anchors append `[diagnostics: <codes>]` when `diagnostic_codes` is declared.
//! - Modular Rust emission honours the root `Target_Crates` projection and preserves legacy topology otherwise.
//!
//! ## 3. Preconditions
//! - `std::env::temp_dir()` is writable for RAII `TempDirGuard` fixtures.
//!
//! ## 4. Postconditions
//! - All temporary directories are reclaimed on drop and every assertion holds.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ over bounded synthetic SysML fixtures.
//! - Space Complexity: $O(N)$ for in-memory artifact maps and emitted manifests.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Exercises `transpile_safety_suite` and `emit_modular_rust_tree`.
//! - State: Isolated per-test temporary workspaces with zero shared mutable state.
//! - Action: Uses `Result` propagation with zero `.unwrap()`, `.expect()`, or `panic!` calls.
//! - Resource: Reclaims every temporary directory deterministically on scope exit.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-STPA-SAFETY-SUITE-RENDER-REGULATORY-OBJECTIVES-ASSESSMENT], [REQ-0607]

use std::collections::BTreeMap;
use std::path::{Component, Path};

use compile_sysml::modular_codegen::{
    domain::{GeneratedTreeManifest, ModularityBounds},
    emitters::rust_emitter::{emit_modular_rust_tree, ModularRustOptions},
};
use compile_sysml::stpa::safety_suite::transpile_safety_suite;
use deap_core::sysml_ast::{
    AttributeDef, CalcDef, ConstraintDef, PackageDef, PartDef, RequirementDef,
};

use crate::reproducer_tests::TempDirGuard;

type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Joins every rendered safety artifact into one searchable buffer.
fn join_artifacts(artifacts: &BTreeMap<String, String>) -> String {
    artifacts.values().cloned().collect::<Vec<_>>().join("\n")
}

/// Builds a requirement definition carrying a declared diagnostic code namespace.
fn requirement_with_diagnostics(name: &str, req_id: &str, codes: &str) -> RequirementDef {
    RequirementDef {
        name: name.to_string(),
        req_id: req_id.to_string(),
        attributes: vec![AttributeDef {
            name: "diagnostic_codes".to_string(),
            type_name: "String".to_string(),
            default_value: Some(format!("\"{codes}\"")),
            doc: None,
        }],
        ..Default::default()
    }
}

#[test]
fn test_requirement_only_package_grounds_lowercase_requirement_and_diagnostics() -> TestResult {
    let pkg = PackageDef {
        name: "RequirementOnlyPkg".to_string(),
        requirement_defs: vec![requirement_with_diagnostics(
            "SafeModeRequirement",
            "REQ-100",
            "E0100",
        )],
        ..Default::default()
    };

    let artifacts = transpile_safety_suite(&pkg, None);
    let joined = join_artifacts(&artifacts);

    assert!(
        !joined.contains("RequirementDef") && !joined.contains("ConstraintDef"),
        "safety artifacts must not leak AST type tokens: {joined}"
    );
    assert!(
        joined.contains("requirement "),
        "safety artifacts must ground requirement anchors in lowercase prose: {joined}"
    );
    assert!(
        joined.contains("E0100"),
        "safety artifacts must preserve the declared requirement diagnostic code: {joined}"
    );
    Ok(())
}

#[test]
fn test_mixed_package_grounds_lowercase_requirement_constraint_and_calc() -> TestResult {
    let pkg = PackageDef {
        name: "MixedPackage".to_string(),
        part_defs: vec![PartDef {
            name: "MixedController".to_string(),
            ..Default::default()
        }],
        requirement_defs: vec![requirement_with_diagnostics(
            "MixedRequirement",
            "REQ-200",
            "E0200",
        )],
        constraint_defs: vec![ConstraintDef {
            name: "MixedConstraint".to_string(),
            expression: "boundedInput >= 0".to_string(),
            ..Default::default()
        }],
        calc_defs: vec![CalcDef {
            name: "MixedCalc".to_string(),
            expression: Some("boundedInput + 1".to_string()),
            ..Default::default()
        }],
        ..Default::default()
    };

    let artifacts = transpile_safety_suite(&pkg, None);
    let joined = join_artifacts(&artifacts);

    for forbidden in ["RequirementDef", "ConstraintDef", "CalcDef"] {
        assert!(
            !joined.contains(forbidden),
            "safety artifacts must not leak AST type token {forbidden}: {joined}"
        );
    }
    for expected in ["requirement ", "constraint ", "calc "] {
        assert!(
            joined.contains(expected),
            "safety artifacts must ground lowercase prose token {expected:?}: {joined}"
        );
    }
    Ok(())
}

/// Parses a root package whose optional `Target_Crates` attribute selects the projected crate.
fn parse_topology_package(with_target_crates: bool) -> Result<PackageDef, Box<dyn std::error::Error>> {
    let target = if with_target_crates {
        "attribute Target_Crates : String = \"crates/compile-sysml, crates/deap-core\";"
    } else {
        ""
    };
    let schema = format!(
        "package RootPkg {{ {target} part def RootNode {{ @Layer(\"Domain\"); attribute count : Integer = 1; }} package nested_pkg {{ part def NestedNode {{ @Layer(\"Domain\"); attribute flag : Boolean = true; }} }} }}"
    );
    compile_sysml::parse_sysml(&schema).map_err(Into::into)
}

/// Asserts every manifest path is a non-empty, traversal-free, backslash-free relative path.
fn assert_safe_manifest_paths(manifest: &GeneratedTreeManifest) {
    for file in &manifest.files {
        let rel = file.relative_path.as_str();
        assert!(!rel.trim().is_empty(), "emitted relative path must not be empty");
        assert!(!rel.contains('\\'), "emitted relative path must not contain backslashes: {rel}");
        assert!(!Path::new(rel).is_absolute(), "emitted relative path must not be absolute: {rel}");
        assert!(
            !Path::new(rel).components().any(|c| matches!(c, Component::ParentDir)),
            "emitted relative path must not traverse parents: {rel}"
        );
    }
}

/// Emits a modular Rust tree into `target_dir` and returns the generated manifest.
fn emit_tree(
    pkg: &PackageDef,
    target_dir: &std::path::Path,
) -> Result<
    compile_sysml::modular_codegen::domain::GeneratedTreeManifest,
    Box<dyn std::error::Error>,
> {
    let options = ModularRustOptions {
        target_dir: target_dir.to_path_buf(),
        dry_run: false,
        bounds: ModularityBounds::canonical(),
    };
    emit_modular_rust_tree(pkg, &options).map_err(|diagnostics| {
        diagnostics
            .into_iter()
            .map(|d| d.to_string())
            .collect::<Vec<_>>()
            .join("\n")
            .into()
    })
}

#[test]
fn test_codegen_topology_projects_target_crate_and_preserves_legacy_layout() -> TestResult {
    let projected_temp = TempDirGuard::new("deap_remediation_863_projected")?;
    let projected_pkg = parse_topology_package(true)?;
    let projected = emit_tree(&projected_pkg, projected_temp.path())?;
    let projected_root = projected_temp.path().join("crates/compile-sysml/src");

    assert!(
        projected_root.join("mod.rs").is_file(),
        "projected topology must emit crates/compile-sysml/src/mod.rs"
    );
    assert!(
        projected_root.join("nested_pkg/mod.rs").is_file(),
        "projected topology must emit crates/compile-sysml/src/nested_pkg/mod.rs"
    );
    assert!(
        projected
            .files
            .iter()
            .all(|f| f.relative_path.starts_with("crates/compile-sysml/src/")),
        "every projected unit path must be rooted under crates/compile-sysml/src/"
    );
    assert_safe_manifest_paths(&projected);

    let legacy_temp = TempDirGuard::new("deap_remediation_863_legacy")?;
    let legacy_pkg = parse_topology_package(false)?;
    let legacy = emit_tree(&legacy_pkg, legacy_temp.path())?;

    assert!(
        legacy_temp.path().join("mod.rs").is_file(),
        "legacy topology must preserve root mod.rs"
    );
    assert!(
        legacy_temp.path().join("nested_pkg/mod.rs").is_file(),
        "legacy topology must preserve nested_pkg/mod.rs"
    );
    assert!(
        legacy.files.iter().any(|f| f.relative_path == "mod.rs")
            && legacy
                .files
                .iter()
                .any(|f| f.relative_path == "nested_pkg/mod.rs"),
        "legacy manifest must retain unprefixed relative paths"
    );
    assert_safe_manifest_paths(&legacy);
    Ok(())
}

#[test]
fn test_codegen_topology_rejects_unsafe_target_crates_paths() -> TestResult {
    let temp = TempDirGuard::new("deap_remediation_863_escape")?;
    for unsafe_value in ["crates/../escape", r"crates\escape", "/tmp/escape", "foo/bar"] {
        let schema = format!(
            "package RootPkg {{ attribute Target_Crates : String = \"{unsafe_value}\"; part def RootNode {{ @Layer(\"Domain\"); attribute count : Integer = 1; }} }}"
        );
        let pkg = compile_sysml::parse_sysml(&schema)?;
        let result = emit_tree(&pkg, temp.path());
        assert!(
            result.is_err(),
            "unsafe Target_Crates value '{unsafe_value}' must fail closed instead of escaping the target directory"
        );
    }
    assert!(
        !temp.path().join("escape").exists(),
        "no path-traversal artifact may be written outside the target directory"
    );
    Ok(())
}

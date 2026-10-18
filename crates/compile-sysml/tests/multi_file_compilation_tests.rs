//! Multi-file SysML v2 compilation and namespace preservation integration tests.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Verifies that multi-file SysML v2 compilation preserves exact top-level package namespaces
//! across AST merging, round-trip serialization, symbol table indexing, semantic validation,
//! and forward Markdown synchronization under DO-178C Level A and DO-330 TQL-4 standards.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Single-package multi-file trees unwrap to the declared domain package root without nesting
//!   inside a synthetic `SysML_Model` wrapper.
//! - Multi-package trees treat `SysML_Model` as a transparent root container during serialization,
//!   symbol table construction, semantic validation, and Epic synchronization.
//!
//! ## 3. Preconditions
//! - Temporary test directories are writable and isolated per test execution.
//!
//! ## 4. Postconditions
//! - All temporary test fixtures are removed upon completion and no workspace state is mutated.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ over fixture file count and AST node count.
//! - Space Complexity: $O(N)$ auxiliary memory for in-memory AST and symbol structures.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Any unexpected `SysML_Model` wrapper leakage across parser, serializer, symbol table,
//!   validator, or forward sync surfaces as a deterministic test failure.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-SYSML-MULTI-FILE-NAMESPACE]

use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use compile_sysml::parser::grammar::parse_sysml_tree;
use compile_sysml::semantic::serializer::SysmlSerializable;
use compile_sysml::semantic::symbols::SymbolTable;
use compile_sysml::semantic::validator::SemanticValidator;
use compile_sysml::sync::forward::{forward_sync_sysml_to_specs, ForwardSyncOptions};
use compile_sysml::parse_sysml;

/// Verifies that multi-file single-package and multi-package SysML schemas do not leak an
/// extraneous `SysML_Model` wrapper across AST parsing, serialization, symbol indexing,
/// semantic validation, or forward specification synchronization.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Ensures deterministic multi-file schema compilation fidelity under DO-178C Level A and
/// DO-330 TQL-4 by preventing synthetic `SysML_Model` namespace pollution across compiler stages.
///
/// ## 2. Invariants & Mathematical Properties
/// - For a single domain package $P$ split across $k \ge 1$ files (including packageless
///   fragments), $\text{parse\_sysml\_tree}(F).\text{name} = P$ and $\text{packages} = \emptyset$.
/// - For $m \ge 2$ distinct top-level packages $\{P_1, \dots, P_m\}$, round-trip serialization
///   omits `package SysML_Model`, `SymbolTable` indexes $P_i::E$ without a `SysML_Model::` prefix,
///   and `forward_sync_sysml_to_specs` generates zero `EPIC-01-SysML_Model.md` artifacts.
///
/// ## 3. Preconditions
/// - The host temporary directory is writable for staging transient `.sysml` fixture files.
///
/// ## 4. Postconditions
/// - Asserts all five multi-file compilation invariants (a) through (e) and removes the
///   temporary fixture directory before returning `Ok(())`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ where $N$ is the total byte size of the multi-file fixtures.
/// - Space Complexity: $O(N)$ bounded by the parsed `PackageDef`, `SymbolTable`, and sync map.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Returns `Err` if file I/O or parsing fails; triggers assertion failure if any compiler stage
///   emits or indexes an extraneous `SysML_Model` wrapper.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-MULTI-FILE-NAMESPACE]
#[test]
fn test_multi_file_single_and_multi_package_no_extraneous_sysml_model_wrapper(
) -> Result<(), Box<dyn std::error::Error>> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_nanos();
    let tmp_dir = std::env::temp_dir().join(format!(
        "deap_multi_file_compilation_test_{}_{}",
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(&tmp_dir)?;

    // (a) Single domain package split across multiple packaged files
    let f1 = tmp_dir.join("part1.sysml");
    let f2 = tmp_dir.join("part2.sysml");
    fs::write(
        &f1,
        "package FlightControl {\n    part def Autopilot {}\n}\n",
    )?;
    fs::write(
        &f2,
        "package FlightControl {\n    part def AttitudeSensor {}\n}\n",
    )?;

    let single_root = parse_sysml_tree(&[f1.clone(), f2])?;
    assert_eq!(single_root.name, "FlightControl");
    assert_eq!(single_root.part_defs.len(), 2);
    assert!(single_root.packages.is_empty());

    // (b) Single domain package split across a packaged file and a packageless fragment file
    let f_frag = tmp_dir.join("fragment.sysml");
    fs::write(&f_frag, "part def RateGyro {}\n")?;
    let mixed_root = parse_sysml_tree(&[f1, f_frag])?;
    assert_eq!(mixed_root.name, "FlightControl");
    assert_eq!(mixed_root.part_defs.len(), 2);
    assert!(mixed_root.packages.is_empty());

    // (c) Multiple top-level domain packages across files: serialization and round-trip re-parse
    let m1 = tmp_dir.join("sub1.sysml");
    let m2 = tmp_dir.join("sub2.sysml");
    fs::write(&m1, "package Sub1 {\n    part def NavEngine {}\n}\n")?;
    fs::write(
        &m2,
        "package Sub2 {\n    part def GuidanceEngine {\n        part nav : Sub1::NavEngine;\n    }\n    requirement def ReqNav {\n        id = \"REQ-NAV-01\";\n        text = \"Navigation engine shall provide guidance solution.\";\n        satisfy by Sub1::NavEngine;\n    }\n}\n",
    )?;

    let multi_root = parse_sysml_tree(&[m1, m2])?;
    let serialized = multi_root.to_sysml(0);
    assert!(
        !serialized.contains("package SysML_Model"),
        "Serialized multi-package schema must not contain extraneous SysML_Model wrapper:\n{}",
        serialized
    );

    let reparsed = parse_sysml(&serialized)?;
    assert_eq!(reparsed.packages.len(), 2);
    assert_eq!(
        reparsed.packages.first().map(|p| p.name.as_str()),
        Some("Sub1")
    );
    assert_eq!(
        reparsed.packages.get(1).map(|p| p.name.as_str()),
        Some("Sub2")
    );

    // (d) SymbolTable and SemanticValidator scope resolution without SysML_Model:: prefix
    let sym_table = SymbolTable::build_from_package(&multi_root);
    assert!(sym_table.lookup("Sub1::NavEngine").is_some());
    assert!(sym_table.lookup("Sub2::GuidanceEngine").is_some());
    assert!(sym_table.lookup("SysML_Model::Sub1::NavEngine").is_none());
    assert!(sym_table
        .lookup("SysML_Model::Sub2::GuidanceEngine")
        .is_none());

    let mut validator = SemanticValidator::new(&multi_root);
    let diagnostics = validator.validate().to_vec();
    assert!(
        !validator.has_errors(),
        "Expected zero semantic validation errors for cross-package reference Sub1::NavEngine, got: {:?}",
        diagnostics
    );

    // (e) forward_sync_sysml_to_specs does not generate phantom epics/EPIC-01-SysML_Model.md
    let out_dir = tmp_dir.join("docs");
    let opts = ForwardSyncOptions {
        schema_path: tmp_dir.join("schema.sysml"),
        docs_dir: out_dir.clone(),
        out_dir: Some(out_dir),
        dry_run: true,
        force: true,
    };
    let generated = forward_sync_sysml_to_specs(&multi_root, &opts)?;
    assert!(
        !generated.contains_key("epics/EPIC-01-SysML_Model.md"),
        "forward_sync_sysml_to_specs must not emit phantom EPIC-01-SysML_Model.md: {:?}",
        generated.keys().collect::<Vec<_>>()
    );

    let _ = fs::remove_dir_all(&tmp_dir);
    Ok(())
}

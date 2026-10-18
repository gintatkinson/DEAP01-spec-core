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

/// Verifies that multi-file schemas containing leading doc comments, line comments, and top-level
/// imports before `package` declarations unwrap synthetic `SysML_Model` roots and preserve exact
/// qualified package scopes when compiled to `.pipeline/schema.sysml`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Prevents Check 31 (`Dual-Schema SSOT Parity`) namespace mismatches under DO-178C Level A and
/// DO-330 TQL-4 when subsystem realization schemas include leading `doc` blocks or `import`
/// clauses prior to their top-level `package` declaration.
///
/// ## 2. Invariants & Mathematical Properties
/// - A single-package file with leading `doc` or `import` unwraps directly to its declared
///   `PackageDef` with merged imports and documentation.
/// - A multi-file schema tree compiled via `parse_sysml_tree` and `to_sysml(0)` never emits an
///   outer `package SysML_Model` wrapper even when top-level `doc` or `import` declarations exist.
/// - Unsorted input file slices to `parse_sysml_tree` produce identical package ordering to sorted
///   slices.
///
/// ## 3. Preconditions
/// - Temporary directory is writable for creating `schema/` and `.pipeline/` test fixtures.
///
/// ## 4. Postconditions
/// - Confirms zero `package SysML_Model` wrapper emission, deterministic file ordering, and
///   round-trip package/definition scope parity before removing the temporary fixture directory.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N \log N)$ for file path sorting plus $O(B)$ for parsing and serialization.
/// - Space Complexity: $O(B)$ bounded by total schema fixture byte size $B$.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Fails deterministically if `parse_package_or_file`, `parse_sysml_tree`, or `PackageDef::to_sysml`
///   wraps top-level packages inside `SysML_Model`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-MULTI-FILE-NAMESPACE]
#[test]
fn test_multi_file_with_top_level_doc_and_imports_preserves_package_scopes_and_dual_schema_parity(
) -> Result<(), Box<dyn std::error::Error>> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_nanos();
    let tmp_dir = std::env::temp_dir().join(format!(
        "deap_multi_file_doc_import_test_{}_{}",
        std::process::id(),
        nanos
    ));
    let schema_dir = tmp_dir.join("schema");
    let pipeline_dir = tmp_dir.join(".pipeline");
    fs::create_dir_all(&schema_dir)?;
    fs::create_dir_all(&pipeline_dir)?;

    let domain_file = schema_dir.join("01_cuas_coa.sysml");
    let cockpit_file = schema_dir.join("02_cuas_coa_cockpit_realization.sysml");
    let solver_file = schema_dir.join("03_cuas_coa_solver_realization.sysml");

    fs::write(
        &domain_file,
        "doc /* Domain architecture model */\n// Header comment\npackage CUAS_COA_System {\n    part def TrackManager {\n        port trackOut;\n    }\n    requirement def ReqTrackLatency {\n        id = \"REQ-TRK-01\";\n        text = \"Track manager shall publish tracks within 50 ms.\";\n        satisfy by TrackManager;\n    }\n}\n",
    )?;

    fs::write(
        &cockpit_file,
        "doc /* Extracted cockpit realization model */\nimport ScalarValues::*;\nimport CUAS_COA_System::*;\n\npackage CockpitRealization {\n    part def TacticalDisplay {\n        attribute refreshHz : Real;\n    }\n    action def RenderTrackOverlay;\n}\n",
    )?;

    fs::write(
        &solver_file,
        "doc /* Multi-package solver and domain extension file */\nimport CUAS_COA_System::*;\n\npackage SolverRealization {\n    part def EngagementSolver {\n        port solutionOut;\n    }\n    constraint def SolverBudgetConstraint;\n}\n\npackage CUAS_COA_System {\n    part def InterceptPlanner {\n        port planOut;\n    }\n}\n",
    )?;

    // 1. Single-package file with leading doc + line comment must preserve doc on CUAS_COA_System
    let domain_parsed = parse_sysml(&fs::read_to_string(&domain_file)?)?;
    assert_eq!(domain_parsed.name, "CUAS_COA_System");
    assert_eq!(
        domain_parsed.doc.as_deref(),
        Some("Domain architecture model")
    );

    // 2. Single-package file with top-level doc and imports must unwrap to CockpitRealization
    let cockpit_parsed = parse_sysml(&fs::read_to_string(&cockpit_file)?)?;
    assert_eq!(
        cockpit_parsed.name, "CockpitRealization",
        "Single-package file with leading doc/imports must unwrap to CockpitRealization, got: {}",
        cockpit_parsed.name
    );
    assert!(
        cockpit_parsed.packages.is_empty(),
        "Unwrapped single-package file must not nest CockpitRealization under packages"
    );
    assert_eq!(cockpit_parsed.imports.len(), 2);

    // 3. Unsorted vs sorted file slices in parse_sysml_tree must be deterministic and identical
    let tree_unsorted = parse_sysml_tree(&[
        solver_file.clone(),
        domain_file.clone(),
        cockpit_file.clone(),
    ])?;
    let tree_sorted = parse_sysml_tree(&[domain_file, cockpit_file, solver_file])?;
    assert_eq!(tree_unsorted, tree_sorted);
    assert!(compile_sysml::parser::grammar::is_synthetic_forest_root(
        &tree_sorted
    ));

    // 4. Serialized .pipeline/schema.sysml must NEVER contain `package SysML_Model` and must preserve top-level doc
    let compiled_sysml = tree_sorted.to_sysml(0);
    let pipeline_schema_path = pipeline_dir.join("schema.sysml");
    fs::write(&pipeline_schema_path, &compiled_sysml)?;

    assert!(
        !compiled_sysml.contains("package SysML_Model"),
        "Compiled .pipeline/schema.sysml must never wrap multi-file packages in SysML_Model:\n{}",
        compiled_sysml
    );
    assert!(
        compiled_sysml.contains("Multi-package solver and domain extension file"),
        "Top-level doc on synthetic forest root must be preserved at indent 0 in serialized output:\n{}",
        compiled_sysml
    );

    // 5. Re-parsing .pipeline/schema.sysml preserves all 3 top-level packages and merged definitions
    let reparsed = parse_sysml(&compiled_sysml)?;
    let pkg_names: Vec<&str> = reparsed.packages.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(
        pkg_names,
        vec!["CUAS_COA_System", "CockpitRealization", "SolverRealization"]
    );

    let sym_table = SymbolTable::build_from_package(&reparsed);
    assert!(sym_table.lookup("CUAS_COA_System::TrackManager").is_some());
    assert!(sym_table.lookup("CUAS_COA_System::InterceptPlanner").is_some());
    assert!(sym_table.lookup("CockpitRealization::TacticalDisplay").is_some());
    assert!(sym_table.lookup("SolverRealization::EngagementSolver").is_some());
    assert!(sym_table.lookup("SysML_Model::CUAS_COA_System::TrackManager").is_none());
    // Root-level import `import CUAS_COA_System::*;` must be resolvable from any child package scope
    assert!(sym_table
        .lookup_in_scope("SolverRealization", "TrackManager")
        .is_some());

    let _ = fs::remove_dir_all(&tmp_dir);
    Ok(())
}


/// Verifies that single-file and multi-file SysML v2 schemas containing top-level `import`
/// declarations before `package` blocks unwrap synthetic `SysML_Model` containers, preserve
/// deterministic package ordering regardless of input slice order, and emit zero outer
/// `package SysML_Model` wrappers during serialization.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Ensures deterministic AST normalization and serialization under DO-178C Level A, DO-330 TQL-4,
/// ISO 26262 ASIL D, and ECSS-E-ST-40C by preventing top-level `import` statements from triggering
/// synthetic `SysML_Model` namespace wrapping in single-file or multi-file compilations.
///
/// ## 2. Invariants & Mathematical Properties
/// - For a single-package source starting with `import ...;` followed by `package P { ... }`,
///   `parse_sysml` returns `PackageDef` with `name == P`, `packages.is_empty()`, and the top-level
///   imports merged into `P.imports`.
/// - For a multi-file schema directory with top-level imports and multiple distinct packages
///   (`CUAS_COA_System`, `CockpitRealization`, `SolverRealization`), `parse_sysml_tree` orders
///   files deterministically regardless of slice permutation and `to_sysml(0)` never emits an
///   enclosing `package SysML_Model { ... }` wrapper.
///
/// ## 3. Preconditions
/// - Host temporary directory is writable for creating isolated `.sysml` test fixtures.
///
/// ## 4. Postconditions
/// - Asserts single-file import unwrapping, multi-file import unwrapping, and permutation
///   invariance, cleaning up temporary fixtures prior to returning `Ok(())`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N log N) over the input file slice length plus O(B) over total byte size.
/// - Space Complexity: O(B) auxiliary memory for parsed `PackageDef` ASTs and serialized strings.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Returns `Err` on fixture I/O or parse errors; panics with diagnostic context if `SysML_Model`
///   wraps any single-file or multi-file package output.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-MULTI-FILE-NAMESPACE]
#[test]
fn test_multi_file_top_level_imports_and_unsorted_slices_unwrap_sysml_model(
) -> Result<(), Box<dyn std::error::Error>> {
    // (a) Single file starting with top-level import before package DomainPkg
    let single_with_import = "import ScalarValues::*;\n\npackage DomainPkg {\n    part def SensorNode {}\n}\n";
    let parsed_single = parse_sysml(single_with_import)?;
    assert_eq!(
        parsed_single.name, "DomainPkg",
        "Single package preceded by top-level import must unwrap to DomainPkg, got: {}",
        parsed_single.name
    );
    assert!(
        parsed_single.packages.is_empty(),
        "Unwrapped DomainPkg must not retain nested package wrappers"
    );
    assert_eq!(
        parsed_single.imports.len(),
        1,
        "Top-level import must be merged into unwrapped DomainPkg"
    );

    // (b) Multi-file schema directory with top-level imports and multiple distinct packages
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_nanos();
    let tmp_dir = std::env::temp_dir().join(format!(
        "deap_multi_file_imports_test_{}_{}",
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(&tmp_dir)?;

    let f_domain = tmp_dir.join("01_cuas_coa.sysml");
    let f_cockpit = tmp_dir.join("02_cuas_coa_cockpit_realization.sysml");
    let f_solver = tmp_dir.join("03_cuas_coa_solver_realization.sysml");

    fs::write(
        &f_domain,
        "import ScalarValues::*;\n\npackage CUAS_COA_System {\n    part def RadarTracker {}\n}\n",
    )?;
    fs::write(
        &f_cockpit,
        "import CUAS_COA_System::*;\n\npackage CockpitRealization {\n    part def TacticalDisplay {}\n}\n",
    )?;
    fs::write(
        &f_solver,
        "package SolverRealization {\n    import CUAS_COA_System::*;\n    part def InterceptSolver {}\n}\n",
    )?;

    // Also test a multi-package file that has a top-level import so root.imports is populated
    let f_shared_imports = tmp_dir.join("00_shared_imports.sysml");
    fs::write(&f_shared_imports, "import ISQ::*;\n")?;

    let tree_forward = parse_sysml_tree(&[
        f_shared_imports.clone(),
        f_domain.clone(),
        f_cockpit.clone(),
        f_solver.clone(),
    ])?;
    let tree_reverse = parse_sysml_tree(&[
        f_solver,
        f_cockpit,
        f_domain,
        f_shared_imports,
    ])?;

    let serialized_forward = tree_forward.to_sysml(0);
    let serialized_reverse = tree_reverse.to_sysml(0);

    assert_eq!(
        serialized_forward, serialized_reverse,
        "parse_sysml_tree must produce identical deterministic output regardless of input slice order"
    );
    assert!(
        !serialized_forward.contains("package SysML_Model"),
        "Multi-file schema with top-level imports must not emit outer package SysML_Model wrapper:\n{}",
        serialized_forward
    );
    assert!(serialized_forward.contains("package CUAS_COA_System {"));
    assert!(serialized_forward.contains("package CockpitRealization {"));
    assert!(serialized_forward.contains("package SolverRealization {"));

    let _ = fs::remove_dir_all(&tmp_dir);
    Ok(())
}


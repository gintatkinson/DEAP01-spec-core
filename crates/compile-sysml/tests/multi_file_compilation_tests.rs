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
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use compile_sysml::parse_sysml;
use compile_sysml::salsa_db::{
    is_synthetic_forest_root, parse_sysml_tree, SemanticValidator, SymbolTable, SysmlSerializable,
};
use compile_sysml::sync::forward::{forward_sync_sysml_to_specs, ForwardSyncOptions};

/// Allocates a collision-free temporary directory for multi-file compilation tests.
/// Realises: [REQ-SYSML-MULTI-FILE-NAMESPACE]
fn make_temp_dir(prefix: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let dir = std::env::temp_dir().join(format!("{prefix}_{}_{nanos}", std::process::id()));
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Verifies multi-package symbol resolution, semantic validation, and forward sync on `multi_root`.
/// Realises: [REQ-SYSML-MULTI-FILE-NAMESPACE]
fn verify_multi_package_symbols_and_sync(
    tmp_dir: &Path,
    multi_root: &deap_core::sysml_cst::PackageDef,
) -> Result<(), Box<dyn std::error::Error>> {
    let sym_table = SymbolTable::build_from_package(multi_root);
    assert!(sym_table.lookup("Sub1::NavEngine").is_some());
    assert!(sym_table.lookup("Sub2::GuidanceEngine").is_some());
    assert!(sym_table.lookup("SysML_Model::Sub1::NavEngine").is_none());
    assert!(sym_table.lookup("SysML_Model::Sub2::GuidanceEngine").is_none());

    let mut validator = SemanticValidator::new(multi_root);
    let diagnostics = validator.validate().to_vec();
    assert!(!validator.has_errors(), "unexpected errors: {diagnostics:?}");

    let out_dir = tmp_dir.join("docs");
    let opts = ForwardSyncOptions {
        schema_path: tmp_dir.join("schema.sysml"),
        docs_dir: out_dir.clone(),
        out_dir: Some(out_dir),
        dry_run: true,
        force: true,
    };
    let generated = forward_sync_sysml_to_specs(multi_root, &opts)?;
    assert!(!generated.contains_key("epics/EPIC-01-SysML_Model.md"));
    Ok(())
}

#[test]
fn test_multi_file_single_and_multi_package_no_extraneous_sysml_model_wrapper(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = make_temp_dir("deap_multi_file_compilation_test")?;
    let f1 = tmp_dir.join("part1.sysml");
    let f2 = tmp_dir.join("part2.sysml");
    fs::write(&f1, "package FlightControl {\n    part def Autopilot {}\n}\n")?;
    fs::write(&f2, "package FlightControl {\n    part def AttitudeSensor {}\n}\n")?;

    let single_root = parse_sysml_tree(&[f1.clone(), f2])?;
    assert_eq!(single_root.name, "FlightControl");
    assert_eq!(single_root.part_defs.len(), 2);
    assert!(single_root.packages.is_empty());

    let f_frag = tmp_dir.join("fragment.sysml");
    fs::write(&f_frag, "part def RateGyro {}\n")?;
    let mixed_root = parse_sysml_tree(&[f1, f_frag])?;
    assert_eq!(mixed_root.name, "FlightControl");
    assert_eq!(mixed_root.part_defs.len(), 2);
    assert!(mixed_root.packages.is_empty());

    let m1 = tmp_dir.join("sub1.sysml");
    let m2 = tmp_dir.join("sub2.sysml");
    fs::write(&m1, "package Sub1 {\n    part def NavEngine {}\n}\n")?;
    fs::write(
        &m2,
        "package Sub2 {\n    part def GuidanceEngine {\n        part nav : Sub1::NavEngine;\n    }\n    requirement def ReqNav {\n        id = \"REQ-NAV-01\";\n        text = \"Navigation engine shall provide guidance solution.\";\n        satisfy by Sub1::NavEngine;\n    }\n}\n",
    )?;

    let multi_root = parse_sysml_tree(&[m1, m2])?;
    let serialized = multi_root.to_sysml(0);
    assert!(!serialized.contains("package SysML_Model"), "wrapper leaked:\n{serialized}");

    let reparsed = parse_sysml(&serialized)?;
    assert_eq!(reparsed.packages.len(), 2);
    assert_eq!(reparsed.packages.first().map(|p| p.name.as_str()), Some("Sub1"));
    assert_eq!(reparsed.packages.get(1).map(|p| p.name.as_str()), Some("Sub2"));

    verify_multi_package_symbols_and_sync(&tmp_dir, &multi_root)?;
    let _ = fs::remove_dir_all(&tmp_dir);
    Ok(())
}

/// Writes the 3 CUAS COA domain/cockpit/solver schema files into `schema_dir`.
/// Realises: [REQ-SYSML-MULTI-FILE-NAMESPACE]
fn write_cuas_coa_fixture_files(
    schema_dir: &Path,
) -> Result<(PathBuf, PathBuf, PathBuf), Box<dyn std::error::Error>> {
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
    Ok((domain_file, cockpit_file, solver_file))
}

#[test]
fn test_multi_file_with_top_level_doc_and_imports_preserves_package_scopes_and_dual_schema_parity(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = make_temp_dir("deap_multi_file_doc_import_test")?;
    let schema_dir = tmp_dir.join("schema");
    let pipeline_dir = tmp_dir.join(".pipeline");
    fs::create_dir_all(&schema_dir)?;
    fs::create_dir_all(&pipeline_dir)?;

    let (domain_file, cockpit_file, solver_file) = write_cuas_coa_fixture_files(&schema_dir)?;

    let domain_parsed = parse_sysml(&fs::read_to_string(&domain_file)?)?;
    assert_eq!(domain_parsed.name, "CUAS_COA_System");
    assert_eq!(domain_parsed.doc.as_deref(), Some("Domain architecture model"));

    let cockpit_parsed = parse_sysml(&fs::read_to_string(&cockpit_file)?)?;
    assert_eq!(cockpit_parsed.name, "CockpitRealization");
    assert!(cockpit_parsed.packages.is_empty());
    assert_eq!(cockpit_parsed.imports.len(), 2);

    let tree_unsorted = parse_sysml_tree(&[solver_file.clone(), domain_file.clone(), cockpit_file.clone()])?;
    let tree_sorted = parse_sysml_tree(&[domain_file, cockpit_file, solver_file])?;
    assert_eq!(tree_unsorted, tree_sorted);
    assert!(is_synthetic_forest_root(&tree_sorted));

    let compiled_sysml = tree_sorted.to_sysml(0);
    fs::write(pipeline_dir.join("schema.sysml"), &compiled_sysml)?;
    assert!(!compiled_sysml.contains("package SysML_Model"));
    assert!(compiled_sysml.contains("Multi-package solver and domain extension file"));

    let reparsed = parse_sysml(&compiled_sysml)?;
    let pkg_names: Vec<&str> = reparsed.packages.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(pkg_names, vec!["CUAS_COA_System", "CockpitRealization", "SolverRealization"]);

    let sym_table = SymbolTable::build_from_package(&reparsed);
    for sym in [
        "CUAS_COA_System::TrackManager",
        "CUAS_COA_System::InterceptPlanner",
        "CockpitRealization::TacticalDisplay",
        "SolverRealization::EngagementSolver",
    ] {
        assert!(sym_table.lookup(sym).is_some(), "missing {sym}");
    }
    assert!(sym_table.lookup("SysML_Model::CUAS_COA_System::TrackManager").is_none());
    assert!(sym_table.lookup_in_scope("SolverRealization", "TrackManager").is_some());

    let _ = fs::remove_dir_all(&tmp_dir);
    Ok(())
}

#[test]
fn test_multi_file_top_level_imports_and_unsorted_slices_unwrap_sysml_model(
) -> Result<(), Box<dyn std::error::Error>> {
    let single_with_import =
        "import ScalarValues::*;\n\npackage DomainPkg {\n    part def SensorNode {}\n}\n";
    let parsed_single = parse_sysml(single_with_import)?;
    assert_eq!(parsed_single.name, "DomainPkg");
    assert!(parsed_single.packages.is_empty());
    assert_eq!(parsed_single.imports.len(), 1);

    let tmp_dir = make_temp_dir("deap_multi_file_imports_test")?;
    let f_domain = tmp_dir.join("01_cuas_coa.sysml");
    let f_cockpit = tmp_dir.join("02_cuas_coa_cockpit_realization.sysml");
    let f_solver = tmp_dir.join("03_cuas_coa_solver_realization.sysml");
    let f_shared_imports = tmp_dir.join("00_shared_imports.sysml");

    fs::write(&f_domain, "import ScalarValues::*;\n\npackage CUAS_COA_System {\n    part def RadarTracker {}\n}\n")?;
    fs::write(&f_cockpit, "import CUAS_COA_System::*;\n\npackage CockpitRealization {\n    part def TacticalDisplay {}\n}\n")?;
    fs::write(&f_solver, "package SolverRealization {\n    import CUAS_COA_System::*;\n    part def InterceptSolver {}\n}\n")?;
    fs::write(&f_shared_imports, "import ISQ::*;\n")?;

    let tree_forward = parse_sysml_tree(&[
        f_shared_imports.clone(),
        f_domain.clone(),
        f_cockpit.clone(),
        f_solver.clone(),
    ])?;
    let tree_reverse = parse_sysml_tree(&[f_solver, f_cockpit, f_domain, f_shared_imports])?;

    let serialized_forward = tree_forward.to_sysml(0);
    let serialized_reverse = tree_reverse.to_sysml(0);
    assert_eq!(serialized_forward, serialized_reverse);
    assert!(!serialized_forward.contains("package SysML_Model"));
    for pkg_decl in [
        "package CUAS_COA_System {",
        "package CockpitRealization {",
        "package SolverRealization {",
    ] {
        assert!(serialized_forward.contains(pkg_decl));
    }

    let _ = fs::remove_dir_all(&tmp_dir);
    Ok(())
}

//! Core forward-synchronization file emission, subpackage traversal, domain cleanliness, realization matrix, and zero-wall-clock determinism unit tests.
//!
//! Safety & Traceability: Realises: [REQ-SYNC-FORWARD-FORWARD-SYNC-SYSML-TO-SPECS]
//! Contract: Verifies deterministic generation of Epics, Features, User Stories, Use Cases, and STPA matrices across single and nested `PackageDef` trees, absence of phantom or hardcoded domain outputs, package-scoped Use Case realization alignment, and 100% absence of wall-clock `Utc::now` across every production `.rs` submodule in `sync/forward/`.
//! Bounds & Faults: Executes pure in-memory forward-sync passes (`dry_run: true`) and compile-time `include_str!` + directory completeness assertions returning `Result<(), Box<dyn std::error::Error>>` with zero panics.

use super::*;
use deap_core::sysml_ast::PartDef;

/// Verifies forward synchronization emits expected Epic, Feature, and STPA matrix paths.
///
/// Realises: [REQ-SYNC-FORWARD-FORWARD-SYNC-SYSML-TO-SPECS]
#[test]
fn test_forward_sync_generates_expected_files() -> Result<(), Box<dyn std::error::Error>> {
    let mut pkg = PackageDef {
        name: "TestSystem".to_string(),
        ..Default::default()
    };

    let p1 = PartDef {
        name: "StructuralSubsystem".to_string(),
        ..Default::default()
    };
    pkg.part_defs.push(p1);

    let opts = ForwardSyncOptions {
        schema_path: PathBuf::from("schema/test.sysml"),
        docs_dir: PathBuf::from("docs"),
        out_dir: None,
        dry_run: true,
        force: true,
    };

    let generated = forward_sync_sysml_to_specs(&pkg, &opts)?;
    assert!(generated.keys().any(|k| k.starts_with("epics/")));
    assert!(generated.keys().any(|k| k.starts_with("features/")));
    assert!(generated.contains_key("safety/STPA_MATRIX.md"));
    Ok(())
}

/// Verifies recursive package traversal projects subpackage Epics, Features, Use Cases, and User Stories.
///
/// Realises: [REQ-SYNC-FORWARD-FORWARD-SYNC-SYSML-TO-SPECS]
#[test]
fn test_recursive_package_traversal_picks_up_subpackage_elements(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut root = PackageDef {
        name: "DEAP_Compiler_System".to_string(),
        ..Default::default()
    };
    let mut sub = PackageDef {
        name: "Subsystem_Alpha".to_string(),
        ..Default::default()
    };
    sub.part_defs.push(PartDef {
        name: "AlphaPart".to_string(),
        ..Default::default()
    });
    sub.use_case_defs.push(deap_core::sysml_ast::UseCaseDef {
        name: "AlphaUseCase".to_string(),
        subject: Some("AlphaPart".to_string()),
        ..Default::default()
    });
    sub.action_defs.push(deap_core::sysml_ast::ActionDef {
        name: "AlphaAction".to_string(),
        ..Default::default()
    });
    root.packages.push(sub);

    let opts = ForwardSyncOptions {
        schema_path: PathBuf::from("schema/test.sysml"),
        docs_dir: PathBuf::from("docs"),
        out_dir: None,
        dry_run: true,
        force: true,
    };

    let generated = forward_sync_sysml_to_specs(&root, &opts)?;
    assert!(
        generated.keys().any(|k| k.contains("Subsystem_Alpha")),
        "Subpackage must generate Epic"
    );
    assert!(
        generated.keys().any(|k| k.contains("AlphaPart")),
        "Subpackage part must generate Feature"
    );
    assert!(
        generated.keys().any(|k| k.contains("AlphaUseCase")),
        "Subpackage use case must generate Use Case"
    );
    assert!(
        generated.keys().any(|k| k.contains("AlphaAction")),
        "Subpackage action must generate User Story"
    );
    Ok(())
}

/// Verifies emitted specifications contain zero hardcoded legacy domain strings.
///
/// Realises: [REQ-SYNC-FORWARD-FORWARD-SYNC-SYSML-TO-SPECS]
#[test]
fn test_zero_hardcoded_domain_strings_exist_in_output_specifications(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut pkg = PackageDef {
        name: "GenericCompiler".to_string(),
        ..Default::default()
    };
    let mut sub = PackageDef {
        name: "ParserSubsystem".to_string(),
        ..Default::default()
    };
    sub.part_defs.push(PartDef {
        name: "LexerEngine".to_string(),
        ..Default::default()
    });
    pkg.packages.push(sub);

    let opts = ForwardSyncOptions {
        schema_path: PathBuf::from("schema/test.sysml"),
        docs_dir: PathBuf::from("docs"),
        out_dir: None,
        dry_run: true,
        force: true,
    };

    let generated = forward_sync_sysml_to_specs(&pkg, &opts)?;
    let forbidden = [
        "CoreController",
        "TelemetrySync",
        "ExecuteAutonomousMission",
        "HandleSafetyFailsafe",
        "SensorSuite",
        "OperatorConsole",
        "SafetyWatchdog",
        "flight parameters",
    ];

    for (path, content) in &generated {
        for bad in &forbidden {
            assert!(
                !content.contains(bad),
                "File '{}' contains forbidden domain string '{}'",
                path,
                bad
            );
        }
    }
    Ok(())
}

/// Verifies empty behavioral collections never emit phantom Use Case or User Story files.
///
/// Realises: [REQ-SYNC-FORWARD-FORWARD-SYNC-SYSML-TO-SPECS]
#[test]
fn test_empty_collections_do_not_emit_phantom_specifications(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut pkg = PackageDef {
        name: "CleanSystem".to_string(),
        ..Default::default()
    };
    pkg.part_defs.push(PartDef {
        name: "CleanPart".to_string(),
        ..Default::default()
    });

    let opts = ForwardSyncOptions {
        schema_path: PathBuf::from("schema/test.sysml"),
        docs_dir: PathBuf::from("docs"),
        out_dir: None,
        dry_run: true,
        force: true,
    };

    let generated = forward_sync_sysml_to_specs(&pkg, &opts)?;
    assert!(
        !generated.keys().any(|k| k.starts_with("use-cases/")),
        "Empty use cases must not emit phantom use-case files"
    );
    assert!(
        !generated.keys().any(|k| k.starts_with("user-stories/")),
        "Empty interactions must not emit phantom user-story files"
    );
    Ok(())
}

/// Verifies Use Case realization matrices scope User Story and Feature links strictly to their owning package.
///
/// Realises: [REQ-SYNC-FORWARD-FORWARD-SYNC-SYSML-TO-SPECS]
#[test]
fn test_use_case_realization_matrix_traceability_alignment(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut root = PackageDef {
        name: "DEAP_Compiler_System".to_string(),
        ..Default::default()
    };

    let mut sub_alpha = PackageDef {
        name: "Subsystem_Alpha".to_string(),
        ..Default::default()
    };
    sub_alpha.part_defs.push(PartDef {
        name: "AlphaEngine".to_string(),
        ..Default::default()
    });
    sub_alpha.action_defs.push(deap_core::sysml_ast::ActionDef {
        name: "AlphaAction".to_string(),
        ..Default::default()
    });
    sub_alpha.use_case_defs.push(UseCaseDef {
        name: "UC_Alpha_Workflow".to_string(),
        ..Default::default()
    });

    let mut sub_beta = PackageDef {
        name: "Subsystem_Beta".to_string(),
        ..Default::default()
    };
    sub_beta.part_defs.push(PartDef {
        name: "BetaEngine".to_string(),
        ..Default::default()
    });
    sub_beta.action_defs.push(deap_core::sysml_ast::ActionDef {
        name: "BetaAction".to_string(),
        ..Default::default()
    });
    sub_beta.use_case_defs.push(UseCaseDef {
        name: "UC_Beta_Workflow".to_string(),
        ..Default::default()
    });

    let mut sub_gamma = PackageDef {
        name: "Subsystem_Gamma".to_string(),
        ..Default::default()
    };
    sub_gamma.part_defs.push(PartDef {
        name: "GammaEngine".to_string(),
        ..Default::default()
    });
    sub_gamma.use_case_defs.push(UseCaseDef {
        name: "UC_Gamma_Workflow".to_string(),
        ..Default::default()
    });

    root.packages.push(sub_alpha);
    root.packages.push(sub_beta);
    root.packages.push(sub_gamma);

    let opts = ForwardSyncOptions {
        schema_path: PathBuf::from("schema/test.sysml"),
        docs_dir: PathBuf::from("docs"),
        out_dir: None,
        dry_run: true,
        force: true,
    };

    let generated = forward_sync_sysml_to_specs(&root, &opts)?;

    let alpha_uc = generated
        .iter()
        .find(|(k, _)| k.contains("UC_Alpha_Workflow"))
        .map(|(_, v)| v)
        .ok_or("expected value is absent")?;
    let beta_uc = generated
        .iter()
        .find(|(k, _)| k.contains("UC_Beta_Workflow"))
        .map(|(_, v)| v)
        .ok_or("expected value is absent")?;
    let gamma_uc = generated
        .iter()
        .find(|(k, _)| k.contains("UC_Gamma_Workflow"))
        .map(|(_, v)| v)
        .ok_or("expected value is absent")?;

    assert!(
        alpha_uc.contains("AlphaAction"),
        "Alpha UC must link to AlphaAction user story"
    );
    assert!(
        alpha_uc.contains("AlphaEngine"),
        "Alpha UC must link to AlphaEngine feature"
    );
    assert!(
        !alpha_uc.contains("BetaAction"),
        "Alpha UC must NOT link to BetaAction"
    );
    assert!(
        !alpha_uc.contains("BetaEngine"),
        "Alpha UC must NOT link to BetaEngine"
    );

    assert!(
        beta_uc.contains("BetaAction"),
        "Beta UC must link to BetaAction user story"
    );
    assert!(
        beta_uc.contains("BetaEngine"),
        "Beta UC must link to BetaEngine feature"
    );
    assert!(
        !beta_uc.contains("AlphaAction"),
        "Beta UC must NOT link to AlphaAction"
    );
    assert!(
        !beta_uc.contains("AlphaEngine"),
        "Beta UC must NOT link to AlphaEngine"
    );

    assert!(
        gamma_uc.contains("GammaEngine"),
        "Gamma UC must link to GammaEngine feature"
    );
    assert!(
        !gamma_uc.contains("AlphaAction"),
        "Gamma UC must NOT cross-wire to AlphaAction"
    );
    assert!(
        !gamma_uc.contains("BetaAction"),
        "Gamma UC must NOT cross-wire to BetaAction"
    );
    Ok(())
}

/// Verifies `ARCH-06`: `forward_sync_sysml_to_specs` and 100% of production `.rs` files in `sync/forward/` contain zero wall-clock timestamp calls and produce bitwise-deterministic outputs.
///
/// Realises: [REQ-SYNC-FORWARD-FORWARD-SYNC-SYSML-TO-SPECS]
#[test]
fn test_arch_06_forward_sync_zero_wall_clock_timestamps() -> Result<(), Box<dyn std::error::Error>>
{
    let mut pkg = PackageDef {
        name: "DeterministicSystem".to_string(),
        ..Default::default()
    };
    pkg.part_defs.push(PartDef {
        name: "DeterministicController".to_string(),
        ..Default::default()
    });

    let opts = ForwardSyncOptions {
        schema_path: PathBuf::from("schema/test.sysml"),
        docs_dir: PathBuf::from("docs"),
        out_dir: None,
        dry_run: true,
        force: true,
    };

    let run1 = forward_sync_sysml_to_specs(&pkg, &opts)?;
    let run2 = forward_sync_sysml_to_specs(&pkg, &opts)?;
    assert_eq!(
        run1, run2,
        "Forward sync outputs must be 100% deterministic across runs"
    );

    let production_sources: &[(&str, &str)] = &[
        ("epic_diagrams.rs", include_str!("epic_diagrams.rs")),
        ("epic_emitter.rs", include_str!("epic_emitter.rs")),
        ("epic_renderer.rs", include_str!("epic_renderer.rs")),
        ("feature_diagrams.rs", include_str!("feature_diagrams.rs")),
        ("feature_emitter.rs", include_str!("feature_emitter.rs")),
        ("feature_layers.rs", include_str!("feature_layers.rs")),
        ("feature_renderer.rs", include_str!("feature_renderer.rs")),
        ("icd_wbs_emitter.rs", include_str!("icd_wbs_emitter.rs")),
        ("merge_and_cleanup.rs", include_str!("merge_and_cleanup.rs")),
        ("mod.rs", include_str!("mod.rs")),
        ("model_indexer.rs", include_str!("model_indexer.rs")),
        ("naming_and_dates.rs", include_str!("naming_and_dates.rs")),
        ("story_allocator.rs", include_str!("story_allocator.rs")),
        ("story_context.rs", include_str!("story_context.rs")),
        ("story_emitter.rs", include_str!("story_emitter.rs")),
        ("story_layers.rs", include_str!("story_layers.rs")),
        ("story_renderer.rs", include_str!("story_renderer.rs")),
        ("use_case_clauses.rs", include_str!("use_case_clauses.rs")),
        ("use_case_emitter.rs", include_str!("use_case_emitter.rs")),
        ("use_case_renderer.rs", include_str!("use_case_renderer.rs")),
        ("wbs_emitter.rs", include_str!("wbs_emitter.rs")),
    ];

    let utc_now_token = "Utc::now";
    let mut inspected_files: BTreeSet<String> = BTreeSet::new();
    for (file_name, src) in production_sources {
        assert!(
            !src.contains(utc_now_token),
            "forward/{file_name} must not invoke wall-clock time"
        );
        inspected_files.insert((*file_name).to_string());
    }

    let forward_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/sync/forward");
    let mut disk_prod_files: BTreeSet<String> = BTreeSet::new();
    for entry in fs::read_dir(&forward_dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.ends_with(".rs") && name != "tests.rs" && !name.starts_with("tests_") {
            disk_prod_files.insert(name);
        }
    }
    assert_eq!(
        inspected_files, disk_prod_files,
        "test_arch_06 must inspect 100% of production .rs files in crates/compile-sysml/src/sync/forward/"
    );

    Ok(())
}

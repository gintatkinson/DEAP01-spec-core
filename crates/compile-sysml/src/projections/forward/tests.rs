#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! # Core Forward-Sync Emission and Realization Matrix Unit Tests
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Realises: [REQ-0604], [REQ-SYNC-FORWARD-FORWARD-SYNC-SYSML-TO-SPECS]
//! Verifies deterministic generation of Epics, Features, User Stories, Use Cases, and STPA
//! matrices across single and nested `PackageDef` trees, absence of phantom or hardcoded domain
//! outputs, and package-scoped Use Case realization alignment.
//!
//! ## 2. Preconditions, Postconditions & Mathematical Invariants
//! - **Preconditions**: In-memory `PackageDef` fixtures are initialized.
//! - **Postconditions**: `forward_sync_sysml_to_specs` returns expected map entries.
//! - **Invariants**: Empty behavioral slices emit zero phantom Use Case or User Story files.
//!
//! ## 3. Failure Modes, Degraded Operation & Recovery
//! - Test assertions return `Result<(), Box<dyn std::error::Error>>` without panicking.
//!
//! ## 4. Concurrency, Memory & Resource Bounds
//! - Pure in-memory `dry_run: true` test execution.
//!
//! ## 5. Dependencies & Integration Contracts
//! - Exercises `forward_sync_sysml_to_specs` and package-scoped realization matrix generation.
//!
//! ## 6. Verification & Acceptance Criteria
//! - Executed as part of the `compile-sysml` unit test suite.
//!
//! ## 7. Realization & Traceability
//! - Realises: [REQ-0604], [REQ-SYNC-FORWARD-FORWARD-SYNC-SYSML-TO-SPECS]

use super::*;
use deap_core::sysml_cst::PartDef;

/// Verifies forward synchronization emits expected Epic, Feature, and STPA matrix paths.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-FORWARD-SYNC-SYSML-TO-SPECS]
#[test]
fn test_forward_sync_generates_expected_files() -> Result<(), Box<dyn std::error::Error>> {
    let mut pkg = PackageDef {
        name: "TestSystem".to_string(),
        ..Default::default()
    };
    pkg.part_defs.push(PartDef {
        name: "StructuralSubsystem".to_string(),
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
    assert!(generated.keys().any(|k| k.starts_with("epics/")));
    assert!(generated.keys().any(|k| k.starts_with("features/")));
    assert!(generated.contains_key("safety/STPA_MATRIX.md"));
    Ok(())
}

/// Verifies recursive package traversal projects subpackage Epics, Features, Use Cases, and User Stories.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-FORWARD-SYNC-SYSML-TO-SPECS]
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
    sub.use_case_defs.push(deap_core::sysml_cst::UseCaseDef {
        name: "AlphaUseCase".to_string(),
        subject: Some("AlphaPart".to_string()),
        ..Default::default()
    });
    sub.action_defs.push(deap_core::sysml_cst::ActionDef {
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
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-FORWARD-SYNC-SYSML-TO-SPECS]
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
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-FORWARD-SYNC-SYSML-TO-SPECS]
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

/// Builds a 3-subsystem test model (`Alpha`, `Beta`, `Gamma`) for Use Case realization matrix tests.
/// Realises: [REQ-0604]
fn build_realization_matrix_test_model() -> PackageDef {
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
    sub_alpha.action_defs.push(ActionDef {
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
    sub_beta.action_defs.push(ActionDef {
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
    root
}

/// Verifies Use Case realization matrices scope User Story and Feature links strictly to their owning package.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-FORWARD-SYNC-SYSML-TO-SPECS]
#[test]
fn test_use_case_realization_matrix_traceability_alignment(
) -> Result<(), Box<dyn std::error::Error>> {
    let root = build_realization_matrix_test_model();
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

    assert!(alpha_uc.contains("AlphaAction") && alpha_uc.contains("AlphaEngine"));
    assert!(!alpha_uc.contains("BetaAction") && !alpha_uc.contains("BetaEngine"));

    assert!(beta_uc.contains("BetaAction") && beta_uc.contains("BetaEngine"));
    assert!(!beta_uc.contains("AlphaAction") && !beta_uc.contains("AlphaEngine"));

    assert!(gamma_uc.contains("GammaEngine"));
    assert!(!gamma_uc.contains("AlphaAction") && !gamma_uc.contains("BetaAction"));
    Ok(())
}

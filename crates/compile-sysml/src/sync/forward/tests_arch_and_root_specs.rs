#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! # Zero Wall-Clock Determinism (`ARCH-06`), Domain Contamination, and Root-Package Projection Tests
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Realises: [REQ-0604], [REQ-SYNC-FORWARD-FORWARD-SYNC-SYSML-TO-SPECS]
//! Verifies 100% absence of wall-clock `Utc::now` calls across every production `.rs` file in
//! `sync/forward/`, zero compiler-domain contamination in downstream specs (`Issue #442`), and
//! root-package behavioral checklist linkage (`Issue #447`).
//!
//! ## 2. Preconditions, Postconditions & Mathematical Invariants
//! - **Preconditions**: `FORWARD_SYNC_PRODUCTION_SOURCES` embeds all production `.rs` files in `sync/forward/`.
//! - **Postconditions**: Confirms 1:1 parity between `FORWARD_SYNC_PRODUCTION_SOURCES` and disk files.
//! - **Invariants**: Repeated runs of `forward_sync_sysml_to_specs` produce identical output maps.
//!
//! ## 3. Failure Modes, Degraded Operation & Recovery
//! - Unlisted production `.rs` files or `Utc::now` occurrences fail the test deterministically.
//!
//! ## 4. Concurrency, Memory & Resource Bounds
//! - Pure in-memory `dry_run: true` execution and read-only directory listing.
//!
//! ## 5. Dependencies & Integration Contracts
//! - Inspects all production submodules in `crates/compile-sysml/src/sync/forward/`.
//!
//! ## 6. Verification & Acceptance Criteria
//! - Executed as part of the `compile-sysml` unit test suite.
//!
//! ## 7. Realization & Traceability
//! - Realises: [REQ-0604], [REQ-SYNC-FORWARD-FORWARD-SYNC-SYSML-TO-SPECS]

use super::*;
use deap_core::sysml_cst::PartDef;

const FORWARD_SYNC_PRODUCTION_SOURCES: &[(&str, &str)] = &[
    ("epic_diagrams.rs", include_str!("epic_diagrams.rs")),
    ("epic_emitter.rs", include_str!("epic_emitter.rs")),
    ("epic_renderer.rs", include_str!("epic_renderer.rs")),
    (
        "escaping_and_dates.rs",
        include_str!("escaping_and_dates.rs"),
    ),
    ("feature_diagrams.rs", include_str!("feature_diagrams.rs")),
    ("feature_emitter.rs", include_str!("feature_emitter.rs")),
    ("feature_layers.rs", include_str!("feature_layers.rs")),
    ("feature_renderer.rs", include_str!("feature_renderer.rs")),
    ("feature_schema.rs", include_str!("feature_schema.rs")),
    ("icd_wbs_emitter.rs", include_str!("icd_wbs_emitter.rs")),
    ("merge_and_cleanup.rs", include_str!("merge_and_cleanup.rs")),
    ("merge_boilerplate.rs", include_str!("merge_boilerplate.rs")),
    ("mod.rs", include_str!("mod.rs")),
    ("model_indexer.rs", include_str!("model_indexer.rs")),
    ("naming_and_dates.rs", include_str!("naming_and_dates.rs")),
    ("story_allocator.rs", include_str!("story_allocator.rs")),
    ("story_context.rs", include_str!("story_context.rs")),
    ("story_emitter.rs", include_str!("story_emitter.rs")),
    ("story_layers.rs", include_str!("story_layers.rs")),
    ("story_renderer.rs", include_str!("story_renderer.rs")),
    ("story_resolver.rs", include_str!("story_resolver.rs")),
    ("use_case_clauses.rs", include_str!("use_case_clauses.rs")),
    ("use_case_diagrams.rs", include_str!("use_case_diagrams.rs")),
    ("use_case_emitter.rs", include_str!("use_case_emitter.rs")),
    ("use_case_renderer.rs", include_str!("use_case_renderer.rs")),
    ("wbs_emitter.rs", include_str!("wbs_emitter.rs")),
    ("wbs_formatters.rs", include_str!("wbs_formatters.rs")),
];

/// Verifies `ARCH-06`: `forward_sync_sysml_to_specs` and 100% of production `.rs` files in `sync/forward/` contain zero wall-clock timestamp calls and produce bitwise-deterministic outputs.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-FORWARD-SYNC-SYSML-TO-SPECS]
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

    let utc_now_token = "Utc::now";
    let mut inspected_files: BTreeSet<String> = BTreeSet::new();
    for (file_name, src) in FORWARD_SYNC_PRODUCTION_SOURCES {
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

/// Verifies Issue `#442` zero compiler-domain contamination across generated downstream specs.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-TEST-FEATURE-BDD-DERIVED]
#[test]
fn test_issue_442_forward_sync_zero_compiler_contamination_in_specs(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut root = PackageDef {
        name: "Satellite_System".to_string(),
        ..Default::default()
    };
    let mut sub = PackageDef {
        name: "Attitude_Control".to_string(),
        ..Default::default()
    };
    sub.part_defs.push(PartDef {
        name: "AttitudeController".to_string(),
        ..Default::default()
    });
    sub.action_defs.push(ActionDef {
        name: "OrientSolarArraysAction".to_string(),
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
    for (path, content) in &generated {
        assert!(
            !content.contains("DEAPCompilerSystem"),
            "File '{}' contains contaminated class DEAPCompilerSystem",
            path
        );
        assert!(
            !content.contains("Compiler Primacy"),
            "File '{}' contains contaminated 'Compiler Primacy' title",
            path
        );
        assert!(
            !content.contains("actor HumanEngineer"),
            "File '{}' contains contaminated 'actor HumanEngineer'",
            path
        );
        assert!(
            !content.contains("FEAT-01-HumanEngineer.md"),
            "File '{}' contains contaminated 'FEAT-01-HumanEngineer.md'",
            path
        );
        assert!(
            !content.contains("initialize_compiler"),
            "File '{}' contains contaminated statechart transition 'initialize_compiler'",
            path
        );
        assert!(
            !content.contains("IngestOEMArtifacts"),
            "File '{}' contains contaminated statechart 'IngestOEMArtifacts'",
            path
        );
    }
    Ok(())
}

/// Builds a root-only `SysML_Model` package fixture for Issue `#447` verification.
/// Realises: [REQ-0604]
fn build_issue_447_root_package_model() -> PackageDef {
    let mut root = PackageDef {
        name: "SysML_Model".to_string(),
        ..Default::default()
    };
    root.part_defs.push(PartDef {
        name: "TrackingRadar".to_string(),
        ..Default::default()
    });
    root.part_defs.push(PartDef {
        name: "MitigationEffector".to_string(),
        ..Default::default()
    });
    root.action_defs.push(ActionDef {
        name: "DetectTrackAction".to_string(),
        ..Default::default()
    });
    root.action_defs.push(ActionDef {
        name: "EngageThreatAction".to_string(),
        ..Default::default()
    });
    root.use_case_defs.push(UseCaseDef {
        name: "DetectTargetUseCase".to_string(),
        ..Default::default()
    });
    root
}

/// Verifies Issue `#447` root-package specification emission, behavioral action stories, and linked Epic checklists.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-TEST-FEATURE-BDD-DERIVED]
#[test]
fn test_issue_447_forward_sync_root_package_specs_actions_and_linked_checklists(
) -> Result<(), Box<dyn std::error::Error>> {
    let root = build_issue_447_root_package_model();
    let opts = ForwardSyncOptions {
        schema_path: PathBuf::from("schema/cuas_coa.sysml"),
        docs_dir: PathBuf::from("docs"),
        out_dir: None,
        dry_run: true,
        force: true,
    };

    let generated = forward_sync_sysml_to_specs(&root, &opts)?;
    assert!(generated.keys().any(|k| k.contains("US-01") && k.contains("DetectTrackAction")));
    assert!(generated.keys().any(|k| k.contains("US-02") && k.contains("EngageThreatAction")));
    assert!(generated.keys().any(|k| k.contains("UC-01") && k.contains("DetectTargetUseCase")));

    let epic_content = generated
        .iter()
        .find(|(k, _)| k.contains("EPIC-01") && k.contains("SysML_Model"))
        .map(|(_, v)| v)
        .ok_or("EPIC-01-SysML_Model.md must be generated")?;

    assert!(!epic_content.contains("Feature 01: Subsystem capability implementation for System Architecture."));
    assert!(epic_content.contains("[Feature 01: [System Architecture] TrackingRadar](../features/FEAT-01-TrackingRadar.md)"));
    assert!(epic_content.contains("[Feature 02: [System Architecture] MitigationEffector](../features/FEAT-02-MitigationEffector.md)"));
    assert!(epic_content.contains("[User Story 01: Detect Track Action](../user-stories/US-01-DetectTrackAction.md)"));
    assert!(epic_content.contains("[Use Case 01: Detect Target Use Case](../use-cases/UC-01-DetectTargetUseCase.md)"));
    Ok(())
}

/// Verifies Issue `#760`: `forward_sync_sysml_to_specs` rejects top-down sequential fake `#NNN` blocks (`#732..#794`) back to `TBD` while preserving bottom-up tracker allocations (`#680 < #690`).
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-RESOLVE-ISSUE-ID]
#[test]
fn test_issue_760_rejects_top_down_sequential_fake_issue_ids(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = std::env::temp_dir().join(format!("deap_issue_760_fwd_{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp_dir);
    for sub in ["epics", "features", "use-cases", "user-stories"] {
        fs::create_dir_all(tmp_dir.join(sub))?;
    }
    fs::write(tmp_dir.join("epics/EPIC-01-VerifiedBottomUpPkg.md"), "| Attribute | Specification Detail |\n| :--- | :--- |\n| **Issue ID** | #690 |\n")?;
    fs::write(tmp_dir.join("features/FEAT-01-VerifiedEngine.md"), "| Attribute | Specification Detail |\n| :--- | :--- |\n| **Issue ID** | #680 |\n")?;
    fs::write(tmp_dir.join("epics/EPIC-02-TopDownFakePkg.md"), "| Attribute | Specification Detail |\n| :--- | :--- |\n| **Issue ID** | #732 |\n")?;
    fs::write(tmp_dir.join("features/FEAT-02-FakePartitioner.md"), "| Attribute | Specification Detail |\n| :--- | :--- |\n| **Issue ID** | #737 |\n")?;
    fs::write(tmp_dir.join("use-cases/UC-01-FakeCycleCheck.md"), "| Attribute | Specification Detail |\n| :--- | :--- |\n| **Issue ID** | #760 |\n")?;
    fs::write(tmp_dir.join("user-stories/US-01-FakeVerifyDagAction.md"), "| Attribute | Specification Detail |\n| :--- | :--- |\n| **Issue ID** | #794 |\n")?;

    let root = PackageDef {
        name: "VerifiedBottomUpPkg".to_string(),
        part_defs: vec![PartDef { name: "VerifiedEngine".to_string(), ..Default::default() }],
        packages: vec![PackageDef {
            name: "TopDownFakePkg".to_string(),
            part_defs: vec![PartDef { name: "FakePartitioner".to_string(), ..Default::default() }],
            action_defs: vec![ActionDef { name: "FakeVerifyDagAction".to_string(), ..Default::default() }],
            use_case_defs: vec![UseCaseDef { name: "FakeCycleCheck".to_string(), ..Default::default() }],
            ..Default::default()
        }],
        ..Default::default()
    };
    let opts = ForwardSyncOptions {
        schema_path: tmp_dir.join("schema/model.sysml"),
        docs_dir: tmp_dir.clone(),
        out_dir: Some(tmp_dir.clone()),
        dry_run: true,
        force: true,
    };
    let gen = forward_sync_sysml_to_specs(&root, &opts)?;
    let _ = fs::remove_dir_all(&tmp_dir);

    let epic1 = gen.get("epics/EPIC-01-VerifiedBottomUpPkg.md").ok_or("missing EPIC-01")?;
    assert!(epic1.contains("| **Issue ID** | #690 |") && epic1.contains("- [ ] #680 - [Feature 01:"));
    let epic2 = gen.get("epics/EPIC-02-TopDownFakePkg.md").ok_or("missing EPIC-02")?;
    assert!(epic2.contains("| **Issue ID** | TBD |") && !epic2.contains("#732") && !epic2.contains("#737") && !epic2.contains("#760") && !epic2.contains("#794"));
    let uc1 = gen.get("use-cases/UC-01-FakeCycleCheck.md").ok_or("missing UC-01")?;
    assert!(uc1.contains("| **Issue ID** | TBD |") && uc1.contains("| **TBD** | User Story |") && uc1.contains("| **TBD** | Feature |"));
    assert_eq!(parse_issue_id_from_content("| **Issue ID** | #690 |\n"), Some(690));
    assert_eq!(
        parse_issue_id_with_provenance("| **Issue ID** | [#690](https://github.com/org/repo/issues/690) |\n"),
        Some(ParsedIssueId { id: 690, has_tracker_url: true })
    );
    Ok(())
}

/// Verifies `resolve_existing_issue_id`, `extract_id_prefix`, `advance_html_comment_state`, `parse_markdown_operation_bullet`, and `escape_json_string` handle multi-byte UTF-8 boundaries cleanly.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-UC-DOC-HELPERS-UNIT]
#[test]
fn test_resolve_existing_issue_id_and_checked_slicing_utf8_boundaries(
) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(extract_id_prefix("UC-01-軌道制御.md"), Some("UC-01-"));
    assert_eq!(extract_id_prefix("軌道-02-Spec.md"), Some("軌道-02-"));
    assert_eq!(extract_id_prefix("UC--"), Some("UC--"));
    assert_eq!(extract_id_prefix("UC-01.md"), None);
    assert_eq!(extract_id_prefix("-"), None);
    assert_eq!(extract_id_prefix(""), None);

    let tmp_dir = std::env::temp_dir().join(format!(
        "deap_test_fwd_sync_utf8_prefix_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&tmp_dir);
    let uc_dir = tmp_dir.join("use-cases");
    fs::create_dir_all(&uc_dir)?;
    fs::write(
        uc_dir.join("UC-07-軌道制御_Legacy.md"),
        "---\nissue_id: 77\n---\n# Legacy Use Case\n",
    )?;

    assert_eq!(
        resolve_existing_issue_id(&tmp_dir, "use-cases/UC-07-軌道制御_Renamed.md"),
        Some(77),
        "Renamed specification sharing 2-dash prefix must preserve existing tracker issue ID"
    );
    assert_eq!(
        resolve_existing_issue_id(&tmp_dir, "use-cases/MalformedSingleDash-Spec.md"),
        None
    );
    assert_eq!(resolve_existing_issue_id(&tmp_dir, "use-cases/-.md"), None);
    let _ = fs::remove_dir_all(&tmp_dir);

    let mut in_comment = false;
    assert!(advance_html_comment_state("<!-- 軌道 ΔΩ", &mut in_comment));
    assert!(in_comment);
    assert!(advance_html_comment_state(
        "still inside --> inline <!-- reopened 🛰",
        &mut in_comment
    ));
    assert!(in_comment);
    assert!(advance_html_comment_state("closed -->", &mut in_comment));
    assert!(!in_comment);

    assert!(parse_markdown_operation_bullet("- `+)broken( : void`").is_none());
    let parsed_op = parse_markdown_operation_bullet("- `+step_solver(in Δt: Real) : SolverState`")
        .ok_or("Expected valid operation bullet to parse")?;
    assert_eq!(parsed_op.name, "step_solver");
    assert_eq!(parsed_op.in_params.len(), 1);

    assert_eq!(escape_json_string("Δ\x07\"\\"), "Δ\\u0007\\\"\\\\");
    Ok(())
}

//! Integration verification test suite for `REQ-0604` (`AC-02`, `AC-03`) and `REQ-0605` (`AC-01`..`AC-04`) (Part F).

/// Verifies `REQ-0604` `AC-02`: Level 1C Logical ICD Matrix (`ICD_01`) and Master Signal Dictionary (`ICD_02`) compilation and `E0200` rejection.
#[test]
fn test_req_0604_ac02_logical_icd_matrix_and_signal_dictionary() {
    use compile_sysml::sync::forward::{
        compile_logical_icd_artifacts, emit_logical_icd_artifacts, try_compile_logical_icd_artifacts,
    };
    use deap_core::sysml_cst::{ConnectionDef, PackageDef, PartDef, PortDef};

    let mut pkg = PackageDef { name: "AvionicsSystem".to_string(), ..Default::default() };
    pkg.part_defs.push(PartDef {
        name: "FlightDirector".to_string(),
        ports: vec![PortDef { name: "cmd_out".to_string(), type_name: "GuidanceCmd".to_string(), direction: "out".to_string(), ..Default::default() }],
        ..Default::default()
    });
    pkg.part_defs.push(PartDef {
        name: "ServoActuator".to_string(),
        ports: vec![PortDef { name: "cmd_in".to_string(), type_name: "GuidanceCmd".to_string(), direction: "in".to_string(), ..Default::default() }],
        ..Default::default()
    });
    pkg.connection_defs.push(ConnectionDef {
        name: "GuidanceLink".to_string(),
        source_part: Some("FlightDirector".to_string()),
        source_port: "cmd_out".to_string(),
        target_part: Some("ServoActuator".to_string()),
        target_port: "cmd_in".to_string(),
        item_flow_ref: Some("GuidanceCmd".to_string()),
        is_flow: true,
        ..Default::default()
    });

    let (icd01, icd02) = compile_logical_icd_artifacts(&pkg);
    assert!(icd01.contains("# ICD_01: System Interface Matrix"));
    assert!(icd01.contains("| **FlightDirector** | --- |"));
    assert!(icd01.contains("GuidanceCmd"));
    assert!(icd02.contains("# ICD_02: Master Signal Dictionary"));
    assert!(icd02.contains("GuidanceCmd"));

    let tmp_dir = std::env::temp_dir().join(format!("deap_req_0604_icd_test_{}", std::process::id()));
    let emitted = emit_logical_icd_artifacts(&pkg, &tmp_dir);
    assert!(emitted.is_ok(), "Expected ICD emission to succeed: {emitted:?}");
    if let Ok((p1, p2)) = emitted {
        assert!(p1.is_file() && p2.is_file());
    }
    std::fs::remove_dir_all(&tmp_dir).unwrap_or(());

    let mut bad_pkg = pkg.clone();
    bad_pkg.connection_defs.push(ConnectionDef {
        name: "BrokenLink".to_string(),
        source_part: Some("FlightDirector".to_string()),
        source_port: "cmd_out".to_string(),
        target_part: Some("GhostSubsystem".to_string()),
        target_port: "missing_port".to_string(),
        ..Default::default()
    });
    let err_res = try_compile_logical_icd_artifacts(&bad_pkg);
    assert!(err_res.as_ref().err().is_some_and(|e| e.contains("Diagnostic: E0200")));

    let empty_pkg = PackageDef { name: "EmptyPkg".to_string(), ..Default::default() };
    let empty_res = try_compile_logical_icd_artifacts(&empty_pkg);
    assert!(empty_res.as_ref().err().is_some_and(|e| e.contains("Diagnostic: E0200")));
}

/// Verifies `REQ-0604` `AC-03`: Level 4 MIL-STD-881E WBS & Enterprise Export compilation, 7-file persistence, and `E0300` rejection on empty models.
#[test]
fn test_req_0604_ac03_wbs_level4_and_enterprise_exports() {
    use compile_sysml::sync::forward::{
        compile_wbs_artifacts, emit_wbs_artifacts, try_compile_wbs_artifacts,
    };
    use deap_core::sysml_cst::{PackageDef, PartDef, RequirementDef, TestCaseDef};

    let mut root = PackageDef {
        name: "MissionSystem".to_string(),
        ..Default::default()
    };
    let mut sub = PackageDef {
        name: "NavigationPkg".to_string(),
        ..Default::default()
    };
    sub.part_defs.push(PartDef {
        name: "InertialSolver".to_string(),
        ..Default::default()
    });
    sub.requirement_defs.push(RequirementDef {
        name: "ReqNavAccuracy".to_string(),
        req_id: "REQ-0101".to_string(),
        ..Default::default()
    });
    sub.test_case_defs.push(TestCaseDef {
        name: "TC_NavAccuracy".to_string(),
        ..Default::default()
    });
    root.packages.push(sub);

    let bundle = compile_wbs_artifacts(&root);
    assert!(bundle.wbs_01_md.contains("01.00.00") && bundle.wbs_01_md.contains("01.01.01"));
    assert!(bundle.wbs_02_md.contains("FEAT-01") && bundle.wbs_02_md.contains("REQ-0101"));
    assert!(bundle.wbs_03_md.contains("Enterprise Export Schema"));
    assert!(bundle.jira_csv.contains("WBS Code") && bundle.monday_csv.contains("WBS Code"));
    assert!(bundle.msproject_xml.contains("<Project"));
    assert!(bundle.ast_json.contains("\"wbs_code\": \"01.01.01\""));

    let tmp_dir =
        std::env::temp_dir().join(format!("deap_req_0604_wbs_test_{}", std::process::id()));
    let written = emit_wbs_artifacts(&root, &tmp_dir);
    assert!(
        written.is_ok(),
        "Expected WBS emission to succeed: {written:?}"
    );
    if let Ok(paths) = written {
        assert_eq!(paths.len(), 7);
        for p in &paths {
            assert!(p.is_file());
        }
    }
    std::fs::remove_dir_all(&tmp_dir).unwrap_or(());

    let empty_pkg = PackageDef {
        name: "EmptySystem".to_string(),
        ..Default::default()
    };
    let err_res = try_compile_wbs_artifacts(&empty_pkg);
    assert!(
        err_res
            .as_ref()
            .err()
            .is_some_and(|e| e.contains("Diagnostic: E0300")),
        "Expected Diagnostic: E0300 on zero PartDef model, got: {err_res:?}"
    );
}

/// Verifies `REQ-0605` `AC-01` and `AC-02`: multi-line Dart `@pragma` annotation 2-pass idempotency and Check 36 audit.
#[test]
fn test_req_0605_ac01_ac02_multiline_dart_annotations() {
    use compile_sysml::docstring::inject_dart_file_docstrings;
    use verify_baseline::checks::docstring_gate::audit_dart_source_docstrings;

    let dart_multiline_annot_src = "\
@pragma(
  'vm:prefer-inline',
)
int computeSignalStep(int currentStep) {
  return currentStep + 1;
}
";
    let pass1_res = inject_dart_file_docstrings(
        dart_multiline_annot_src,
        "app_flutter/lib/signal_step.dart",
        None,
        false,
    );
    assert!(pass1_res.is_ok());
    let (pass1_out, pass1_count) = pass1_res.unwrap_or_default();
    assert_eq!(pass1_count, 1);
    let doc_pos = pass1_out.find("/// ## 1. Safety Intent");
    let pragma_pos = pass1_out.find("@pragma(");
    assert!(doc_pos.is_some() && pragma_pos.is_some() && doc_pos < pragma_pos);

    let pass2_res =
        inject_dart_file_docstrings(&pass1_out, "app_flutter/lib/signal_step.dart", None, false);
    assert!(pass2_res.is_ok());
    let (pass2_out, pass2_count) = pass2_res.unwrap_or_default();
    assert_eq!(pass2_count, 0);
    assert_eq!(pass2_out, pass1_out);
    assert_eq!(
        audit_dart_source_docstrings(&pass1_out, "app_flutter/lib/signal_step.dart"),
        Ok(1)
    );
}

/// Verifies `REQ-0605` `AC-03` and `AC-04`: strict `owner_name` schema isolation so unmapped impl owners never inherit package-level items.
#[test]
fn test_req_0605_ac03_ac04_strict_owner_schema_isolation() {
    use compile_sysml::docstring::{
        collect_all_parts, inject_rust_file_docstrings, resolve_schema_context_strict,
    };
    use deap_core::sysml_cst::{ConstraintDef, PackageDef, PartDef, RequirementDef};
    use verify_baseline::checks::docstring_gate::audit_rust_source_docstrings;

    let schema_pkg = PackageDef {
        name: "CoreControlPkg".to_string(),
        part_defs: vec![PartDef {
            name: "MappedDispatcher".to_string(),
            is_def: true,
            satisfied_requirements: vec!["REQ-MAPPED-01".to_string()],
            ..Default::default()
        }],
        requirement_defs: vec![RequirementDef {
            name: "process_frame".to_string(),
            req_id: "REQ-PKG-LEAK-999".to_string(),
            ..Default::default()
        }],
        constraint_defs: vec![ConstraintDef {
            name: "invariant_process_frame_1".to_string(),
            expression: "frame_seq > 0".to_string(),
            is_assertion: true,
            ..Default::default()
        }],
        ..Default::default()
    };

    let unmapped_owner_rust = "\
pub struct UnmappedHelper;

impl UnmappedHelper {
    pub fn process_frame(&self, frame_seq: u32) -> u32 {
        frame_seq
    }
}
";
    let mut parts_out = Vec::new();
    collect_all_parts(&schema_pkg, &mut parts_out);
    assert_eq!(parts_out.len(), 1);
    assert_eq!(
        resolve_schema_context_strict(Some(&schema_pkg), Some("UnmappedHelper"), "process_frame"),
        (None, None)
    );
    assert_eq!(
        resolve_schema_context_strict(
            Some(&schema_pkg),
            Some("MappedDispatcher"),
            "process_frame"
        ),
        (
            Some("REQ-PKG-LEAK-999".to_string()),
            Some("invariant_process_frame_1 (frame_seq > 0)".to_string()),
        )
    );
    let rust_res = inject_rust_file_docstrings(
        unmapped_owner_rust,
        "crates/core_module/src/unmapped_helper.rs",
        Some(&schema_pkg),
        false,
    );
    assert!(rust_res.is_ok());
    let (rust_enriched, rust_count) = rust_res.unwrap_or_default();
    assert_eq!(rust_count, 2);
    assert!(!rust_enriched.contains("REQ-PKG-LEAK-999") && !rust_enriched.contains("frame_seq > 0"));
    assert_eq!(
        audit_rust_source_docstrings(
            &rust_enriched.replace("Realises: [", "[DOC-TIER: A] Realises: ["),
            "crates/core_module/src/unmapped_helper.rs"
        ),
        Ok(2)
    );
}

/// Verifies `REQ-0604` parent Epic helpers, HTML comment re-open transitions, and inclusive Epic counting.
#[test]
fn test_req_0604_ac04_html_comment_reopen_and_inclusive_epic_count() {
    use compile_sysml::sync::forward::{
        advance_html_comment_state, collect_package_story_numbers, count_total_epics_inclusive,
        execute_emit_cli_args, resolve_parent_epic_link, StoryMeta,
    };
    use deap_core::sysml_cst::PackageDef;

    let pkg_root = PackageDef { name: "SystemRoot".to_string(), ..Default::default() };
    let pkg_conops = PackageDef { name: "ConOps".to_string(), ..Default::default() };
    let pkg_sub = PackageDef { name: "GuidanceSubsystem".to_string(), ..Default::default() };
    let ordered = vec![&pkg_root, &pkg_conops, &pkg_sub];
    assert_eq!(resolve_parent_epic_link(&ordered, "SystemRoot"), "[EPIC-01-SystemRoot](../epics/EPIC-01-SystemRoot.md)");
    assert_eq!(resolve_parent_epic_link(&ordered, "ConOps"), "[EPIC-02-ConOps](../epics/EPIC-02-ConOps.md)");
    assert_eq!(resolve_parent_epic_link(&ordered, "GuidanceSubsystem"), "[EPIC-03-GuidanceSubsystem](../epics/EPIC-03-GuidanceSubsystem.md)");

    let stories = vec![
        StoryMeta { pkg_name: "GuidanceSubsystem".to_string(), s_num: 1, ..Default::default() },
        StoryMeta { pkg_name: "OtherSubsystem".to_string(), s_num: 2, ..Default::default() },
        StoryMeta { pkg_name: "GuidanceSubsystem".to_string(), s_num: 3, ..Default::default() },
    ];
    assert_eq!(collect_package_story_numbers(&stories, "GuidanceSubsystem"), vec![1, 3]);

    let mut in_comment = false;
    assert!(advance_html_comment_state("<!-- start", &mut in_comment) && in_comment);
    assert!(advance_html_comment_state("end --> mid <!-- reopen", &mut in_comment) && in_comment);
    assert!(advance_html_comment_state("end --> mid <!-- closed -->", &mut in_comment) && !in_comment);

    let child_specs = vec![
        ("epics/EPIC-02-ConOps.md".to_string(), "# Epic 2".to_string()),
        ("epics/EPIC-03-Nav.md".to_string(), "# Epic 3".to_string()),
        ("epics/EPIC-02-ConOps.md".to_string(), "# Epic 2 dup".to_string()),
        ("features/FEAT-01-Core.md".to_string(), "# Feat 1".to_string()),
    ];
    assert_eq!(count_total_epics_inclusive("epics/EPIC-01-Root.md", true, &child_specs), 3);
    assert_eq!(count_total_epics_inclusive("features/FEAT-01-Core.md", false, &child_specs), 2);

    let normal_args = vec!["compile-sysml".to_string(), "schema/model.sysml".to_string()];
    assert_eq!(execute_emit_cli_args(&normal_args), Ok(None));
}

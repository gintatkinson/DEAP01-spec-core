//! Unit tests (part D) for `parity_gates`: Issue #440 Check 29/30A/30B/30C error formatting and Issues #537/#538 discovery.
//!
//! Safety & Traceability: Realises: [REQ-PARITY-GATE-29], [REQ-COVERAGE-DIGEST-GATE], [REQ-SYSML-MODEL-DISCOVERY]
//! Contract: Exercises error prefix formatting for Checks 29/30A/30B/30C and `docs/designs/*.sysml` model discovery.
//! Bounds & Faults: Uses auto-cleaned temporary directories (`TempDir`); zero shared mutable state.

use super::parity_check_a_tests::TempDir;
use super::parity_gates::*;
use std::fs;

#[test]
fn test_issue_440_research_inventory_io_error_formats_as_check_29(
) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let tmp = TempDir::new("res_inv_io")?;
        let epics_dir = tmp.path.join("docs").join("epics");
        fs::create_dir_all(&epics_dir)?;
        fs::write(epics_dir.join("EPIC-001.md"), "# Epic 1\n")?;
        let unreadable = epics_dir.join("forbidden");
        fs::create_dir(&unreadable)?;
        let mut perms = fs::metadata(&unreadable)?.permissions();
        perms.set_mode(0o000);
        fs::set_permissions(&unreadable, perms)?;

        let res = check_research_inventory(&tmp.path, false, false);

        let mut restore_perms = fs::metadata(&unreadable)?.permissions();
        restore_perms.set_mode(0o755);
        let _ = fs::set_permissions(&unreadable, restore_perms);

        let errs = res.err().unwrap_or_default();
        assert!(
            !errs.is_empty(),
            "Expected I/O errors for unreadable directory"
        );
        let io_err = errs
            .iter()
            .find(|e| e.contains("I/O error walking directory"))
            .ok_or("Expected directory traversal I/O error in error list")?;
        assert!(
            io_err.starts_with("Check 29 failed:"),
            "I/O error must be formatted as Check 29 failed, got: {}",
            io_err
        );
        assert!(
            !io_err.contains("Check 28 failed"),
            "I/O error must not contain legacy 'Check 28 failed': {}",
            io_err
        );
    }
    Ok(())
}

#[test]
fn test_issue_440_coverage_digest_errors_format_as_check_30a(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("cov_digest")?;
    let feat_dir = tmp.path.join("docs").join("features");
    fs::create_dir_all(&feat_dir)?;
    fs::write(feat_dir.join("FEAT-001.md"), "# Feature 1\n")?;

    let reports_dir = tmp.path.join("docs").join("reports");
    fs::create_dir_all(&reports_dir)?;
    fs::write(
        reports_dir.join("coverage.md"),
        "# Coverage\nReferences FEAT-999.md here.\n",
    )?;

    let res = check_coverage_digest(&tmp.path);
    let errs = res.err().unwrap_or_default();
    assert!(!errs.is_empty(), "Expected errors for undeclared spec");
    for err in &errs {
        assert!(
            err.starts_with("Check 30A failed:"),
            "Coverage digest error must be formatted as Check 30A failed, got: {}",
            err
        );
        assert!(
            !err.contains("Check 28 failed"),
            "Coverage digest error must not contain Check 28 failed: {}",
            err
        );
    }
    Ok(())
}

#[test]
fn test_issue_440_obligation_witness_errors_format_as_check_30b(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("ob_witness")?;
    let safety_dir = tmp.path.join("docs").join("safety");
    fs::create_dir_all(&safety_dir)?;
    fs::write(
        safety_dir.join("hazard_analysis.md"),
        "# Hazard Analysis\nDeclared hazard H-001 with no safety constraint.\n",
    )?;

    let res = check_obligation_witness(&tmp.path);
    let errs = res.err().unwrap_or_default();
    assert!(!errs.is_empty(), "Expected errors for unwitnessed hazard");
    for err in &errs {
        assert!(
            err.starts_with("Check 30B failed:"),
            "Obligation witness error must be formatted as Check 30B failed, got: {}",
            err
        );
        assert!(
            !err.contains("Check 29 failed"),
            "Obligation witness error must not contain Check 29 failed: {}",
            err
        );
    }
    Ok(())
}

#[test]
fn test_issue_440_architecture_viewpoint_errors_format_as_check_30c(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("arch_views")?;
    fs::create_dir_all(tmp.path.join(".pipeline").join("upstream"))?;
    fs::write(tmp.path.join("Cargo.toml"), "[workspace]\n")?;

    let res = check_architecture_viewpoint_diagrams(&tmp.path, false, true);
    let errs = res.err().unwrap_or_default();
    assert!(!errs.is_empty(), "Expected error for missing arch views");
    for err in &errs {
        assert!(
            err.starts_with("Check 30C failed:"),
            "Architecture viewpoint error must be formatted as Check 30C failed, got: {}",
            err
        );
        assert!(
            !err.contains("Gate 30 failed") && !err.contains("Check 30 failed"),
            "Architecture viewpoint error must not contain Gate/Check 30 failed: {}",
            err
        );
    }
    Ok(())
}

#[test]
fn test_discover_sysml_model_text_falls_back_to_docs_designs_sysml(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("issue_537_docs_designs_sysml")?;
    let schema_dir = tmp.path.join("schema");
    fs::create_dir_all(&schema_dir)?;
    fs::write(schema_dir.join(".gitkeep"), "")?;

    let designs_dir = tmp.path.join("docs").join("designs");
    fs::create_dir_all(&designs_dir)?;
    fs::write(
        designs_dir.join("blueprint-02-beta.sysml"),
        "package Beta { part def EngineB; }\n",
    )?;
    fs::write(
        designs_dir.join("blueprint-01-alpha.sysml"),
        "package Alpha { part def EngineA; }\n",
    )?;
    fs::write(
        designs_dir.join("blueprint-01-alpha.md"),
        "# Blueprint 01 Alpha\nMarkdown content should be ignored.\n",
    )?;
    fs::write(
        designs_dir.join("blueprint-01-alpha.digest.json"),
        "{\"ignored\": true}\n",
    )?;

    let model_opt = discover_sysml_model_text(&tmp.path);
    assert!(
        model_opt.is_some(),
        "discover_sysml_model_text must fall back to docs/designs/*.sysml when schema/ only has .gitkeep and .pipeline/schema.sysml is absent"
    );
    let text = model_opt.unwrap_or_default();
    assert!(
        text.contains("part def EngineA"),
        "Discovered text must contain 'part def EngineA', got: {}",
        text
    );
    assert!(
        text.contains("part def EngineB"),
        "Discovered text must contain 'part def EngineB', got: {}",
        text
    );
    let pos_a = text.find("part def EngineA").unwrap_or(usize::MAX);
    let pos_b = text.find("part def EngineB").unwrap_or(0);
    assert!(
        pos_a < pos_b,
        "blueprint-01-alpha.sysml must appear before blueprint-02-beta.sysml in sorted order"
    );
    assert!(
        !text.contains("Markdown content should be ignored") && !text.contains("\"ignored\""),
        "Discovered text must ignore .md and .digest.json files in docs/designs/"
    );
    Ok(())
}

#[test]
fn test_discover_architectural_sysml_model_text_includes_docs_designs_sysml(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("issue_538_arch_docs_designs_sysml")?;
    let schema_dir = tmp.path.join("schema");
    fs::create_dir_all(&schema_dir)?;
    fs::write(schema_dir.join(".gitkeep"), "")?;

    let designs_dir = tmp.path.join("docs").join("designs");
    fs::create_dir_all(&designs_dir)?;
    fs::write(
        designs_dir.join("blueprint-01-alpha.sysml"),
        "package Alpha { part def ArchController; }\npackage alpha_realization { part def CodeImpl; }\n",
    )?;
    fs::write(
        designs_dir.join("blueprint-01-alpha.md"),
        "# Blueprint 01 Alpha\nMarkdown content should be ignored.\n",
    )?;
    fs::write(
        designs_dir.join("blueprint-01-alpha.digest.json"),
        "{\"ignored\": true}\n",
    )?;

    let model_opt = discover_architectural_sysml_model_text(&tmp.path);
    assert!(
        model_opt.is_some(),
        "discover_architectural_sysml_model_text must include docs/designs/*.sysml when schema/ only has .gitkeep and .pipeline/schema.sysml is absent"
    );
    let text = model_opt.unwrap_or_default();
    assert!(
        text.contains("part def ArchController"),
        "Discovered architectural text must contain 'part def ArchController', got: {}",
        text
    );
    assert!(
        !text.contains("part def CodeImpl"),
        "Discovered architectural text must strip 'package alpha_realization', got: {}",
        text
    );
    assert!(
        !text.contains("Markdown content should be ignored") && !text.contains("\"ignored\""),
        "Discovered architectural text must ignore .md and .digest.json files in docs/designs/"
    );

    fs::write(
        schema_dir.join("domain.sysml"),
        "package Domain { part def DomainPlant; }\n",
    )?;
    let combined_opt = discover_architectural_sysml_model_text(&tmp.path);
    assert!(
        combined_opt.is_some(),
        "discover_architectural_sysml_model_text must return Some(text) when both schema/domain.sysml and docs/designs/blueprint-01-alpha.sysml exist"
    );
    let combined_text = combined_opt.unwrap_or_default();
    assert!(
        combined_text.contains("part def DomainPlant"),
        "Combined architectural text must contain 'part def DomainPlant', got: {}",
        combined_text
    );
    assert!(
        combined_text.contains("part def ArchController"),
        "Combined architectural text must contain 'part def ArchController', got: {}",
        combined_text
    );
    assert!(
        !combined_text.contains("part def CodeImpl"),
        "Combined architectural text must strip 'package alpha_realization', got: {}",
        combined_text
    );
    Ok(())
}

use super::struct_check_a_tests::TempDir;
use super::structural::*;
use deap_core::rules::CanonicalArchitectureBlueprint;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

#[test]
fn test_reproduce_defect_318_stale_backlog_hallucinations_and_attribution_errors(
) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .ok_or("repo root")?;
    let res = check_handoff_and_backlog_integrity(repo_root);
    assert!(
        res.is_ok(),
        "Defect #318 integrity check failed on repository root: {:?}",
        res.err()
    );
    Ok(())
}

#[test]
fn test_defect_318_mutation_rejects_stale_root_handoff() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("c318_root_handoff")?;
    let handoff = tmp.path.join("HANDOFF.md");
    File::create(&handoff)?;
    let res = check_handoff_and_backlog_integrity(&tmp.path);
    assert!(res.is_err(), "Must reject existence of root HANDOFF.md");
    Ok(())
}

#[test]
fn test_defect_318_mutation_rejects_stale_attribution_error(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("c318_stale_attr")?;
    let designs = tmp.path.join("docs").join("designs");
    fs::create_dir_all(&designs)?;
    let sol = designs.join("feat-289-290-293-solution.md");
    let mut f = File::create(&sol)?;
    writeln!(
        f,
        "# Solution (refs #318)\nprompt quarantine across all skills"
    )?;
    let res = check_handoff_and_backlog_integrity(&tmp.path);
    assert!(res.is_err(), "Must reject stale attribution error");
    Ok(())
}

#[test]
fn test_defect_318_mutation_rejects_stale_work_package_queue(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("c318_stale_queue")?;
    let designs = tmp.path.join("docs").join("designs");
    fs::create_dir_all(&designs)?;
    let sol = designs.join("feat-289-290-293-solution.md");
    let mut f = File::create(&sol)?;
    writeln!(f, "# Solution (refs #318)\nWork Package B: Issue #293")?;
    let res = check_handoff_and_backlog_integrity(&tmp.path);
    assert!(
        res.is_err(),
        "Must reject listing resolved issues as outstanding work packages"
    );
    Ok(())
}

#[test]
fn test_reproduce_defect_321_root_master_handover_checkpoint_rejected(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("c321_root_rejection")?;
    fs::create_dir_all(tmp.path.join(".pipeline/upstream"))?;
    let schema_dir = tmp.path.join("schema");
    fs::create_dir_all(&schema_dir)?;
    File::create(schema_dir.join(".gitkeep"))?;

    let checkpoint = tmp.path.join("MASTER_HANDOVER_CHECKPOINT.md");
    File::create(&checkpoint)?;

    assert!(check_no_master_handover_checkpoint(&tmp.path).is_err());
    assert!(verify_upstream_blueprint_domain_cleanliness(&tmp.path).is_err());
    assert!(check_handoff_and_backlog_integrity(&tmp.path).is_err());
    Ok(())
}

#[test]
fn test_defect_321_docs_master_handover_checkpoint_rejected(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("c321_docs_rejection")?;
    fs::create_dir_all(tmp.path.join(".pipeline/upstream"))?;
    let schema_dir = tmp.path.join("schema");
    fs::create_dir_all(&schema_dir)?;
    File::create(schema_dir.join(".gitkeep"))?;

    let docs_dir = tmp.path.join("docs").join("architecture");
    fs::create_dir_all(&docs_dir)?;
    let checkpoint = docs_dir.join("MASTER_HANDOVER_CHECKPOINT.md");
    File::create(&checkpoint)?;

    assert!(check_no_master_handover_checkpoint(&tmp.path).is_err());
    assert!(verify_upstream_blueprint_domain_cleanliness(&tmp.path).is_err());
    assert!(check_handoff_and_backlog_integrity(&tmp.path).is_err());
    Ok(())
}

#[test]
fn test_defect_321_root_checkpoint_variant_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("c321_variant_rejection")?;
    let variant = tmp.path.join("MASTER_HANDOVER_CHECKPOINT_V2.md");
    File::create(&variant)?;
    assert!(check_no_master_handover_checkpoint(&tmp.path).is_err());
    Ok(())
}

#[test]
fn test_defect_321_current_repo_passes() -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .ok_or("repo root")?;
    assert!(check_no_master_handover_checkpoint(repo_root).is_ok());
    assert!(verify_upstream_blueprint_domain_cleanliness(repo_root).is_ok());
    assert!(check_handoff_and_backlog_integrity(repo_root).is_ok());
    Ok(())
}

#[test]
fn test_issue_441_documentation_contains_no_retired_script_references(
) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .ok_or("repo root")?;
    let res = check_zero_retired_scripts_in_documentation(repo_root);
    assert!(
        res.is_ok(),
        "Documentation must not reference retired scripts (refs #441): {:?}",
        res.err()
    );
    Ok(())
}

#[test]
fn test_defect_441_mutation_rejects_retired_script_in_markdown(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("c441_script_rejection")?;
    let docs = tmp.path.join("docs").join("operations");
    fs::create_dir_all(&docs)?;
    let guide = docs.join("install-guide.md");
    let mut f = File::create(&guide)?;
    let needle = "create_issue.sh";
    writeln!(f, "# Guide\nRun {needle} now")?;

    let res = check_zero_retired_scripts_in_documentation(&tmp.path);
    assert!(res.is_err(), "Must reject retired script reference in docs");
    let errs = res.err().ok_or("expected Err")?;
    assert!(errs.iter().any(|e| e.contains(needle)));
    Ok(())
}

#[test]
fn test_defect_450_retired_python_in_pipeline_docs_fails() -> Result<(), Box<dyn std::error::Error>>
{
    let tmp = TempDir::new("c450_pipeline_retired")?;
    let pipeline_dir = tmp.path.join(".pipeline");
    fs::create_dir_all(&pipeline_dir)?;
    let const_file = pipeline_dir.join("constitution.md");
    let mut f = File::create(&const_file)?;
    writeln!(
        f,
        "# Constitution\nRuns validators/spec_validator.py for quality gate"
    )?;

    let res = check_zero_retired_scripts_in_documentation(&tmp.path);
    assert!(
        res.is_err(),
        "Must reject retired Python validator in .pipeline/constitution.md"
    );
    let errs = res.err().ok_or("expected Err")?;
    assert!(errs.iter().any(|e| e.contains("spec_validator.py")));
    Ok(())
}

#[test]
fn test_defect_459_feat42_design_blueprint_compliance() -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .ok_or("repo root")?;
    let res = check_feat42_design_blueprint_compliance(repo_root);
    assert!(
        res.is_ok(),
        "FEAT-42 design blueprint must comply (refs #459): {:?}",
        res.err()
    );
    Ok(())
}

#[test]
fn test_defect_459_mutation_rejects_sysml_core_in_design() -> Result<(), Box<dyn std::error::Error>>
{
    let tmp = TempDir::new("c459_sysml_core_mutation")?;
    let designs = tmp.path.join("docs").join("designs");
    fs::create_dir_all(&designs)?;
    let feat = designs.join("FEAT-42_source_code_ast_ingestion_and_docstring_synthesizer.md");
    let mut f = File::create(&feat)?;
    writeln!(f, "# FEAT-42\nparticipant Core as sysml-core Parser")?;

    let res = check_feat42_design_blueprint_compliance(&tmp.path);
    assert!(res.is_err());
    let errs = res.err().ok_or("expected Err")?;
    assert!(errs.iter().any(|e| e.contains("sysml-core")));
    Ok(())
}

#[test]
fn test_check_37_blueprint_grounding_passes_on_live_repo() -> Result<(), Box<dyn std::error::Error>>
{
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .ok_or("repo root")?;
    let res = check_blueprint_grounding(repo_root);
    assert!(
        res.is_ok(),
        "Check 37 blueprint grounding failed on live repository: {:?}",
        res.err()
    );
    Ok(())
}

#[test]
fn test_check_37_rejects_fictional_crates_and_nonexistent_paths(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("c37_mutations")?;
    fs::create_dir_all(tmp.path.join(".pipeline").join("upstream"))?;
    let bp_dir = tmp
        .path
        .join("docs")
        .join("architecture")
        .join("blueprints");
    fs::create_dir_all(&bp_dir)?;
    fs::create_dir_all(tmp.path.join("crates").join("deap-core").join("src"))?;
    File::create(
        tmp.path
            .join("crates")
            .join("deap-core")
            .join("src")
            .join("lib.rs"),
    )?;

    for bp in CanonicalArchitectureBlueprint::ALL {
        let mut f = File::create(bp_dir.join(bp.as_str()))?;
        writeln!(f, "# Blueprint\nGrounded in `crates/deap-core/src/lib.rs`.")?;
    }
    assert!(check_blueprint_grounding(&tmp.path).is_ok());

    let target_bp = bp_dir
        .join(CanonicalArchitectureBlueprint::DeapSysmlV2IngestionEngineBlueprint.as_str());
    let mut f = File::create(&target_bp)?;
    writeln!(
        f,
        "# Corrupted Blueprint\nUses `deap-ast` and `deap-harness`.\nRuns `verify_downstream`.\nInspects `crates/verify-baseline/src/checks/uml.rs`.\nContains em dash \u{2014} here."
    )?;

    let err_res = check_blueprint_grounding(&tmp.path);
    assert!(err_res.is_err());
    let errs = err_res.err().ok_or("expected Err")?;
    assert!(errs.iter().any(|e| e.contains("deap-ast")));
    assert!(errs.iter().any(|e| e.contains("deap-harness")));
    assert!(errs.iter().any(|e| e.contains("verify_downstream")));
    assert!(errs
        .iter()
        .any(|e| e.contains("crates/verify-baseline/src/checks/uml.rs")));
    assert!(errs
        .iter()
        .any(|e| e.contains("\\u2014") || e.contains("em dash")));
    Ok(())
}

#[test]
fn test_check_37_scans_docs_designs_blueprints() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("c37_docs_designs_blueprints")?;
    let designs_dir = tmp.path.join("docs").join("designs");
    fs::create_dir_all(&designs_dir)?;

    let bp_file = designs_dir.join("blueprint-01-core.md");
    fs::write(
        &bp_file,
        "# Design Blueprint 01\nGrounded in `crates/deap-parser/src/lib.rs`.\n",
    )?;
    let err_res = check_blueprint_grounding(&tmp.path);
    assert!(err_res.is_err());
    let errs = err_res.err().ok_or("expected Err")?;
    assert!(errs
        .iter()
        .any(|e| e.contains("docs/designs/blueprint-01-core.md") && e.contains("deap-parser")));

    fs::write(
        &bp_file,
        "# Design Blueprint 01\nPure domain design specification without ungrounded crate paths.\n",
    )?;
    assert!(check_blueprint_grounding(&tmp.path).is_ok());
    Ok(())
}

#[test]
fn test_check_37_exempts_downstream_from_canonical_architecture_blueprints(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("c37_downstream_blueprints_dir")?;
    let bp_dir = tmp
        .path
        .join("docs")
        .join("architecture")
        .join("blueprints");
    fs::create_dir_all(&bp_dir)?;
    File::create(bp_dir.join(".gitkeep"))?;

    let designs_dir = tmp.path.join("docs").join("designs");
    fs::create_dir_all(&designs_dir)?;
    let design_bp = designs_dir.join("blueprint-01-core.md");
    fs::write(
        &design_bp,
        "# Design Blueprint 01\nPure downstream domain design specification.\n",
    )?;
    assert!(check_blueprint_grounding(&tmp.path).is_ok());

    fs::write(
        &design_bp,
        "# Design Blueprint 01\nReferences fictional `crates/deap-parser/src/lib.rs`.\n",
    )?;
    assert!(check_blueprint_grounding(&tmp.path).is_err());
    Ok(())
}

use super::*;
use deap_core::workspace::is_upstream_compiler;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[test]
fn test_runner_clean_landing_zones() -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .ok_or("repo root")?;

    let is_clean = has_clean_landing_zones(repo_root);
    let spec_zones = [
        repo_root.join("docs").join("epics"),
        repo_root.join("docs").join("features"),
    ];
    let has_specs = spec_zones.iter().any(|d| {
        d.is_dir()
            && WalkDir::new(d).into_iter().filter_map(|e| e.ok()).any(|e| {
                e.file_type().is_file() && e.file_name().to_string_lossy().ends_with(".md")
            })
    });

    if has_specs {
        assert!(!is_clean || is_upstream_compiler(repo_root));
    } else {
        assert!(is_clean);
    }
    Ok(())
}

#[test]
fn test_runner_check_34_zero_regex() -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .ok_or("repo root")?;
    let res = check_zero_regex(repo_root);
    assert!(
        res.is_ok(),
        "Check 34 Zero Regex Invariant failed on repo: {:?}",
        res.err()
    );
    Ok(())
}

#[test]
fn test_issue_440_runner_check_identifiers_unique() -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .ok_or("repo root")?;
    let runner_src =
        fs::read_to_string(repo_root.join("crates/verify-baseline/src/runner.rs"))?;

    let mut check_identifiers = Vec::new();
    for line in runner_src.lines() {
        if let Some(pos) = line.find("write_defect_dossier(") {
            let rest = line.get(pos..).unwrap_or("");
            if let Some(first_quote) = rest.find('"') {
                let after_first = rest.get(first_quote + 1..).unwrap_or("");
                if let Some(second_quote) = after_first.find('"') {
                    let full_check_name = after_first.get(..second_quote).unwrap_or("");
                    if let Some(paren) = full_check_name.find(" (") {
                        let check_id = full_check_name.get(..paren).unwrap_or("");
                        check_identifiers.push(check_id.to_string());
                    }
                }
            }
        }
    }

    let post_27: Vec<&String> = check_identifiers
        .iter()
        .skip_while(|id| *id != "Check 28")
        .take_while(|id| *id != "Check 35")
        .collect();

    let mut seen = std::collections::BTreeSet::new();
    let mut duplicates = Vec::new();
    for id in &post_27 {
        if !seen.insert(*id) {
            duplicates.push((*id).clone());
        }
    }

    assert!(
        duplicates.is_empty(),
        "Duplicate check identifiers found in runner.rs write_defect_dossier calls: {:?}",
        duplicates
    );

    assert!(
        !runner_src.contains("\"Check 28 (Coverage digest)\""),
        "runner.rs must not label Coverage Digest as Check 28"
    );
    assert!(
        !runner_src.contains("\"Check 29 (Obligation witness)\""),
        "runner.rs must not label Obligation Witness as Check 29"
    );
    assert!(
        !runner_src.contains("\"Check 30 (Architecture viewpoint diagrams)\""),
        "runner.rs must not label Architecture Viewpoint as Check 30"
    );
    assert!(
        runner_src.contains("\"Check 30A (Coverage digest)\""),
        "runner.rs must label Coverage Digest as Check 30A"
    );
    assert!(
        runner_src.contains("\"Check 30B (Obligation witness)\""),
        "runner.rs must label Obligation Witness as Check 30B"
    );
    assert!(
        runner_src.contains("\"Check 30C (Architecture viewpoint diagrams)\""),
        "runner.rs must label Architecture Viewpoint as Check 30C"
    );
    Ok(())
}

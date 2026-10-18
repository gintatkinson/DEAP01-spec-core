//! Integration tests for Check 33 (`check_zero_python_scripts`):
//! Zero Python Scripts & Operational Python/Pytest Reference Invariant (refs #469).

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use verify_baseline::checks::check_zero_python_scripts;

struct ScopedTempWorkspace {
    root: PathBuf,
}

impl ScopedTempWorkspace {
    fn new(prefix: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let root = std::env::temp_dir().join(format!(
            "deap_check33_{}_{}_{}",
            prefix,
            std::process::id(),
            nanos
        ));
        fs::create_dir_all(&root).expect("create temp workspace");
        Self { root }
    }
}

impl Drop for ScopedTempWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn test_check_33_rejects_python_and_pytest_references_in_markdown_yaml_and_json() {
    let ws = ScopedTempWorkspace::new("rejects_operational_refs");

    let skill_dir = ws.root.join("skills").join("example");
    fs::create_dir_all(&skill_dir).expect("create skills/example");
    fs::write(
        skill_dir.join("SKILL.md"),
        "# Example Skill\nRun verification:\npython3 -m pytest tests/\n",
    )
    .expect("write SKILL.md");

    let pipeline_dir = ws.root.join(".pipeline");
    fs::create_dir_all(&pipeline_dir).expect("create .pipeline");
    fs::write(
        pipeline_dir.join(".gitlab-ci.yml"),
        "stages:\n  - verify\nverify_job:\n  script:\n    - python3 scripts/verify.py\n",
    )
    .expect("write .gitlab-ci.yml");
    fs::write(
        pipeline_dir.join("config.json"),
        "{\n  \"runner\": \"pytest\"\n}\n",
    )
    .expect("write config.json");

    let docs_dir = ws.root.join("docs");
    fs::create_dir_all(&docs_dir).expect("create docs");
    fs::write(
        docs_dir.join("guide.md"),
        "# Simulation Guide\nExecute `models/python/sim_engine.py` to validate.\n",
    )
    .expect("write guide.md");

    let res = check_zero_python_scripts(&ws.root);
    assert!(
        res.is_err(),
        "Check 33 must reject operational Python and Pytest references in Markdown, YAML, and JSON files"
    );
    let errs = res.unwrap_err().join("\n");
    assert!(
        errs.contains("skills/example/SKILL.md:3"),
        "Expected violation citing skills/example/SKILL.md:3, got:\n{}",
        errs
    );
    assert!(
        errs.contains(".pipeline/.gitlab-ci.yml:5"),
        "Expected violation citing .pipeline/.gitlab-ci.yml:5, got:\n{}",
        errs
    );
    assert!(
        errs.contains(".pipeline/config.json:2"),
        "Expected violation citing .pipeline/config.json:2, got:\n{}",
        errs
    );
    assert!(
        errs.contains("docs/guide.md:2"),
        "Expected violation citing docs/guide.md:2, got:\n{}",
        errs
    );
}

#[test]
fn test_check_33_allows_governance_prohibition_clauses_and_defect_dossiers() {
    let ws = ScopedTempWorkspace::new("allows_prohibitions_and_defects");

    fs::write(
        ws.root.join("AGENTS.md"),
        "# Agent Governance\n- Zero Python Scripts Invariant (Check 33): `.py` scripts and `pytest` are strictly prohibited.\n- Never invoke `python3` or retired `.py` helpers.\n",
    )
    .expect("write AGENTS.md");

    let rules_dir = ws.root.join("rules");
    fs::create_dir_all(&rules_dir).expect("create rules");
    fs::write(
        rules_dir.join("toolchain.md"),
        "# Toolchain Rules\nLegacy `models/python` and `unittest` runners were eliminated and forbidden under Check 33.\n",
    )
    .expect("write rules/toolchain.md");

    let pipeline_dir = ws.root.join(".pipeline");
    let defects_dir = pipeline_dir.join("defects");
    fs::create_dir_all(&defects_dir).expect("create .pipeline/defects");
    fs::write(
        pipeline_dir.join("ACTIVE_RULES_BUNDLE.md"),
        "# Active Rules Bundle\nWe ban all `.py` files and `python3` invocations across the repository.\n",
    )
    .expect("write ACTIVE_RULES_BUNDLE.md");
    fs::write(
        defects_dir.join("DEFECT_ZERO_PYTHON_REFERENCES.md"),
        "# Defect Dossier\nSymptom: `python3 -m pytest` and `models/python/sim_engine.py` were found in docs.\n",
    )
    .expect("write defect dossier");

    let res = check_zero_python_scripts(&ws.root);
    assert!(
        res.is_ok(),
        "Expected governance prohibition clauses and .pipeline/defects/ dossiers to pass Check 33, got: {:?}",
        res.err()
    );
}

#[test]
fn test_live_workspace_has_zero_operational_python_references() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("resolve repository workspace root");

    let res = check_zero_python_scripts(workspace_root);
    assert!(
        res.is_ok(),
        "Expected live repository workspace to have zero operational Python/Pytest references under Check 33, got:\n{}",
        res.err().unwrap_or_default().join("\n")
    );
}

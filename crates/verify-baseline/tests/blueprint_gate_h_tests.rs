//! Integration verification test suite for `REQ-0606` (`AC-04`..`AC-06`) and Check 37 blueprint grounding (Part H).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const MAX_MODULE_WALK_DEPTH: usize = 16;

/// Resolves a target `.rs` file or its decomposed `<stem>/` module directory into sorted `.rs` paths.
fn resolve_module_rs_files(path: &Path) -> Vec<PathBuf> {
    if path.is_file() {
        return vec![path.to_path_buf()];
    }
    let dir_path = if path.is_dir() {
        path.to_path_buf()
    } else {
        path.with_extension("")
    };
    if !dir_path.is_dir() {
        return Vec::new();
    }
    let mut collected = BTreeSet::new();
    let mut stack = vec![(dir_path, 0usize)];
    while let Some((current_dir, depth)) = stack.pop() {
        if depth > MAX_MODULE_WALK_DEPTH {
            continue;
        }
        let Ok(entries) = std::fs::read_dir(&current_dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let entry_path = entry.path();
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_dir() {
                let next_depth = depth.saturating_add(1);
                if next_depth <= MAX_MODULE_WALK_DEPTH {
                    stack.push((entry_path, next_depth));
                }
            } else if file_type.is_file()
                && entry_path.extension().and_then(|ext| ext.to_str()) == Some("rs")
            {
                collected.insert(entry_path);
            }
        }
    }
    collected.into_iter().collect()
}

/// RAII temporary workspace directory fixture that deterministically removes its directory tree on drop.
struct TempWorkspace {
    root: PathBuf,
}

impl TempWorkspace {
    fn new(prefix: &str) -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let root =
            std::env::temp_dir().join(format!("deap_{}_{}_{}", prefix, std::process::id(), nanos));
        let _ = std::fs::create_dir_all(&root);
        Self { root }
    }

    fn write_file(&self, rel_path: &str, contents: &str) -> PathBuf {
        let full = self.root.join(rel_path);
        if let Some(parent) = full.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&full, contents);
        full
    }
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        if self.root.exists() {
            std::fs::remove_dir_all(&self.root).unwrap_or(());
        }
    }
}

/// Reads all synthesized `DEFECT-*.md` files from `defects_dir` into a deterministically ordered map.
fn collect_defect_dossiers(defects_dir: &Path) -> std::collections::BTreeMap<String, Vec<u8>> {
    let mut dossiers = std::collections::BTreeMap::new();
    if let Ok(entries) = std::fs::read_dir(defects_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let file_name = entry.file_name().to_string_lossy().into_owned();
            if file_name.starts_with("DEFECT-") && file_name.ends_with(".md") {
                if let Ok(bytes) = std::fs::read(&path) {
                    dossiers.insert(file_name, bytes);
                }
            }
        }
    }
    dossiers
}

/// Verifies `REQ-0606` `AC-04`: relative subdirectory & single-file `--path` resolution, library `run_audit_code`, and fail-closed CLI exit codes (`1` and `2`).
#[test]
fn req_0606_ac04_relative_paths_and_fail_closed_cli_probes() {
    use std::process::Command;

    let ws = TempWorkspace::new("req_0606_ac04_rel");
    let _ = std::fs::create_dir_all(ws.root.join(".git").join("objects"));
    let _ = std::fs::create_dir_all(ws.root.join(".git").join("refs"));
    ws.write_file(".git/HEAD", "ref: refs/heads/main\n");
    ws.write_file(".git/config", "[core]\n\trepositoryformatversion = 0\n\tfilemode = true\n\tbare = false\n[remote \"origin\"]\n\turl = https://github.com/sample-org/sample-repo.git\n");
    let defects_dir = ws.root.join(".pipeline").join("defects");
    let _ = std::fs::create_dir_all(&defects_dir);
    ws.write_file("Cargo.toml", "[workspace]\nmembers = [\"crates/sample_crate\"]\n");
    ws.write_file("crates/sample_crate/Cargo.toml", "[package]\nname = \"sample_crate\"\nversion = \"0.1.0\"\nedition = \"2021\"\n");
    ws.write_file("crates/sample_crate/src/lib.rs", "/// Untraced.\npub fn untraced_step(x: u32) -> u32 { x }\n");

    let bin = env!("CARGO_BIN_EXE_verify-baseline");
    for rel_subtarget in ["crates/sample_crate", "crates/sample_crate/src/lib.rs"] {
        for file_name in collect_defect_dossiers(&defects_dir).keys() {
            let _ = std::fs::remove_file(defects_dir.join(file_name));
        }
        if let Ok(rel_out) = Command::new(bin).current_dir(&ws.root).args(["audit-code", "--path", rel_subtarget, "--dry-run"]).output() {
            assert_eq!(rel_out.status.code(), Some(0));
        }
        assert!(!ws.root.join("crates/.pipeline").exists());
        assert!(!ws.root.join("crates/sample_crate/.pipeline").exists());
        let rel_dossiers = collect_defect_dossiers(&defects_dir);
        assert!(!rel_dossiers.is_empty());
        for (file_name, bytes) in &rel_dossiers {
            assert!(String::from_utf8_lossy(bytes).contains("FILE_LOCATION: crates/sample_crate/src/lib.rs:"));
            if let Ok(check_out) = Command::new(bin).current_dir(&ws.root).args(["check-defect", &format!(".pipeline/defects/{file_name}"), "--dry-run"]).output() {
                assert_eq!(check_out.status.code(), Some(0));
            }
        }
    }

    for file_name in collect_defect_dossiers(&defects_dir).keys() {
        let _ = std::fs::remove_file(defects_dir.join(file_name));
    }
    assert!(verify_baseline::audit_code::run_audit_code(
        &ws.root,
        &verify_baseline::audit_code::AuditCodeArgs {
            path: PathBuf::from("crates/sample_crate"),
            pillar: None, issue: None, repo: None, publish: false, dry_run: true,
        },
    ).is_ok());
    assert!(!collect_defect_dossiers(&defects_dir).is_empty());

    let missing_str = ws.root.join("non_existent_audit_target");
    if let Ok(neg1) = Command::new(bin).args(["audit-code", "--path", missing_str.to_str().unwrap_or(""), "--dry-run"]).output() {
        assert_eq!(neg1.status.code(), Some(1));
    }

    let broken_ws = TempWorkspace::new("req_0606_unbalanced_brace");
    let _ = std::fs::create_dir_all(broken_ws.root.join(".pipeline").join("defects"));
    broken_ws.write_file("crates/broken_crate/src/lib.rs", "pub fn malformed_block() {\n    if true {\n");
    if let Ok(neg2) = Command::new(bin).args(["audit-code", "--path", broken_ws.root.to_str().unwrap_or(""), "--dry-run"]).output() {
        assert_eq!(neg2.status.code(), Some(1));
    }
    if let Ok(neg3) = Command::new(bin).args(["audit-code", "--path", ws.root.to_str().unwrap_or(""), "--pillar", "invalid-pillar", "--dry-run"]).output() {
        assert_eq!(neg3.status.code(), Some(2));
    }
}

/// Verifies `REQ-0606` `AC-05` and `AC-06`: `--dry-run` offline isolation under an empty `PATH`
/// and end-to-end 12-check blueprint gate verification on `blueprint-06`.
#[test]
fn req_0606_ac05_and_ac06_dry_run_offline_isolation_and_bp06_blueprint_gate() {
    use std::process::Command;

    let clean_ws = TempWorkspace::new("req_0606_clean_offline");
    assert!(std::fs::create_dir_all(clean_ws.root.join(".pipeline").join("defects")).is_ok());
    clean_ws.write_file(
        "Cargo.toml",
        "[workspace]\nmembers = [\"crates/clean_sample\"]\n",
    );
    clean_ws.write_file(
        "crates/clean_sample/Cargo.toml",
        "[package]\nname = \"clean_sample\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );

    let clean_rust = r#"/// Computes saturating addition of two bounded counters.
///
/// Realises: [REQ-0606/AC-05]
pub fn add_bounded(lhs: u32, rhs: u32) -> u32 {
    lhs.saturating_add(rhs)
}

/// Verifies `add_bounded` nominal saturating sum.
#[test]
fn verify_add_bounded() {
    assert_eq!(add_bounded(2, 3), 5);
}
"#;
    clean_ws.write_file("crates/clean_sample/src/lib.rs", clean_rust);

    let bin = env!("CARGO_BIN_EXE_verify-baseline");
    let clean_ws_str = clean_ws.root.to_str().unwrap_or("");
    assert!(!clean_ws_str.is_empty());

    if let Ok(offline_out) = Command::new(bin)
        .args(["audit-code", "--path", clean_ws_str, "--dry-run"])
        .env("PATH", "")
        .output()
    {
        assert_eq!(offline_out.status.code(), Some(0));
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .unwrap_or(manifest_dir.as_path());
    if let Ok(bp06_out) = Command::new(bin)
        .current_dir(repo_root)
        .args([
            "check-blueprint",
            "docs/designs/blueprint-06-deterministic-static-defect-dossier-auditor.md",
            "--dry-run",
        ])
        .output()
    {
        assert_eq!(
            bp06_out.status.code(),
            Some(0),
            "Expected blueprint-06 check-blueprint --dry-run to exit Some(0), got {:?}; stdout:\n{}\nstderr:\n{}",
            bp06_out.status.code(),
            String::from_utf8_lossy(&bp06_out.stdout),
            String::from_utf8_lossy(&bp06_out.stderr)
        );
    }
}

/// Verifies Check 37 (`verify_baseline::checks::structural::check_blueprint_grounding`) fails closed
/// when an `Approved` design blueprint references a non-existent `crates/...` path, permits `Draft`
/// blueprints, and passes once the referenced module exists on disk.
#[test]
fn check_37_rejects_approved_design_blueprint_with_nonexistent_crates_path() {
    use verify_baseline::checks::structural::check_blueprint_grounding;

    let ws = TempWorkspace::new("check_37_approved_bp");
    ws.write_file("crates/verify-baseline/src/lib.rs", "// existing crate root\n");

    let approved_bp = r#"| Attribute | Specification Detail |
| :--- | :--- |
| **Blueprint ID** | `BP-99-UNIMPLEMENTED` |
| **Requirement ID** | `REQ-BP-099` |
| **Title** | Approved Blueprint Grounding Probe |
| **Status** | `Approved` |
| **Target Crates** | `verify-baseline` |
| **Tracked Issues** | `#691` |
| **Generation Mode** | `subagent` |
| **Specification Source** | `crates/verify-baseline/src/lib.rs` |

## 1. Executive Summary & Derivation-Annotation Contract

Anchored to `crates/verify-baseline/src/lib.rs`.

## 6. Proposed Deterministic Implementation Contract

Target module path: `crates/verify-baseline/src/missing_unimplemented_module.rs`.

## 7. Source References

- `crates/verify-baseline/src/lib.rs`
"#;
    ws.write_file("docs/designs/blueprint-99-unimplemented.md", approved_bp);

    let approved_missing_res = check_blueprint_grounding(&ws.root);
    assert!(approved_missing_res.is_err());
    if let Err(errs) = &approved_missing_res {
        assert!(format!("{errs:?}").contains("missing_unimplemented_module.rs"));
    }

    let draft_bp = approved_bp.replace("| **Status** | `Approved` |", "| **Status** | `Draft` |");
    ws.write_file("docs/designs/blueprint-99-unimplemented.md", &draft_bp);
    assert!(check_blueprint_grounding(&ws.root).is_ok());

    ws.write_file("docs/designs/blueprint-99-unimplemented.md", approved_bp);
    ws.write_file(
        "crates/verify-baseline/src/missing_unimplemented_module.rs",
        "// implemented module\n",
    );
    assert!(check_blueprint_grounding(&ws.root).is_ok());
}

/// Verifies `resolve_module_rs_files` resolves both a single `foo.rs` file and a decomposed `foo/` module directory.
#[test]
fn test_resolve_module_rs_files_single_file_and_decomposed_directory() {
    let ws = TempWorkspace::new("resolve_mod_rs");
    ws.write_file("src/foo.rs", "// single file\n");
    assert_eq!(
        resolve_module_rs_files(&ws.root.join("src/foo.rs")),
        vec![ws.root.join("src/foo.rs")]
    );

    let _ = std::fs::remove_file(ws.root.join("src/foo.rs"));
    ws.write_file("src/foo/sub.rs", "// submodule\n");
    ws.write_file("src/foo/mod.rs", "// mod root\n");
    ws.write_file("src/foo/notes.txt", "ignored\n");
    assert_eq!(
        resolve_module_rs_files(&ws.root.join("src/foo.rs")),
        vec![
            ws.root.join("src/foo/mod.rs"),
            ws.root.join("src/foo/sub.rs"),
        ]
    );
}

/// Verifies `BP-02` (`#750`) retires all `structural.rs` / `struct_check_*.rs` shards and enforces `ra_ap_syntax` CST gates (`[REQ-0602]`).
#[test]
fn bp02_retires_struct_check_shards_and_enforces_cst_visitors() -> Result<(), String> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| String::from("missing root"))?;
    let checks_dir = root.join("crates/verify-baseline/src/checks");
    for retired in [
        "structural.rs",
        "struct_check_audit.rs",
        "struct_check_feat42.rs",
        "struct_check_grounding.rs",
        "struct_check_handoff.rs",
        "struct_check_mask.rs",
        "struct_check_purity.rs",
        "struct_check_scripts.rs",
        "struct_check_upstream.rs",
        "struct_check_a_tests.rs",
        "struct_check_b_tests.rs",
        "struct_check_c_tests.rs",
        "struct_check_d_tests.rs",
        "struct_check_e_tests.rs",
        "struct_check_f_tests.rs",
    ] {
        if checks_dir.join(retired).exists() {
            return Err(format!("retired shard still exists on disk: {retired}"));
        }
    }
    let manifest_ast = deap_core::cargo_manifest_ast::parse_cargo_toml_ast(
        &root.join("Cargo.toml"),
        &root,
    )?;
    if !manifest_ast.is_workspace_root || !manifest_ast.forbidden_dependencies.is_empty() {
        return Err(String::from("unexpected root Cargo.toml AST state"));
    }
    let lock_ast = deap_core::cargo_manifest_ast::parse_cargo_lock_ast(&root.join("Cargo.lock"))?;
    if lock_ast.package_count == 0 || !lock_ast.forbidden_packages.is_empty() {
        return Err(String::from("unexpected Cargo.lock AST state"));
    }
    verify_baseline::checks::cargo_policy_gate::verify_workspace_cargo_policy(&root)
        .map_err(|e| e.join("; "))?;
    Ok(())
}

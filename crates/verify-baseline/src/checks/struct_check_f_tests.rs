use super::struct_check_a_tests::{join_pair, TempDir};
use super::structural::*;
use std::fs::{self, File};
use std::path::Path;

#[test]
fn test_enclosing_upstream_compiler_root_resolves_dogfood_and_fails_closed_standalone(
) -> Result<(), Box<dyn std::error::Error>> {
    let root_tmp = TempDir::new("upstream_dogfood_root")?;
    let dogfood_dir = root_tmp.path.join(".pipeline").join("upstream");
    fs::create_dir_all(&dogfood_dir)?;

    fs::write(root_tmp.path.join(".gitignore"), "target/\n")?;
    fs::write(
        root_tmp.path.join("README.md"),
        "# Upstream Compiler\n## Operator Prompt Catalog\n",
    )?;
    fs::write(root_tmp.path.join("AGENTS.md"), "# Agent Governance\n")?;
    let rules_dir = root_tmp.path.join("rules");
    fs::create_dir_all(&rules_dir)?;
    fs::write(
        rules_dir.join("sysml-ssot-completeness.md"),
        "# SysML SSOT Completeness\n",
    )?;
    let reconcile_crate = root_tmp.path.join("crates").join("reconcile-backlog");
    fs::create_dir_all(reconcile_crate.join("src"))?;
    fs::write(
        reconcile_crate.join("Cargo.toml"),
        "[package]\nname = \"reconcile-backlog\"\nversion = \"0.1.0\"\n",
    )?;
    fs::write(
        reconcile_crate.join("src").join("main.rs"),
        "fn main() {}\n",
    )?;

    assert_eq!(
        enclosing_upstream_compiler_root(&dogfood_dir),
        Some(root_tmp.path.clone())
    );
    assert_eq!(enclosing_upstream_compiler_root(&root_tmp.path), None);

    assert!(check_gitignore_exists(&dogfood_dir).is_ok());
    assert!(check_downstream_instructions_exist(&dogfood_dir).is_ok());
    assert!(check_reconcile_backlog_tooling_exists(&dogfood_dir).is_ok());

    let standalone_tmp = TempDir::new("standalone_downstream_fail_closed")?;
    assert_eq!(enclosing_upstream_compiler_root(&standalone_tmp.path), None);
    assert!(check_gitignore_exists(&standalone_tmp.path).is_err());
    assert!(check_downstream_instructions_exist(&standalone_tmp.path).is_err());
    assert!(check_reconcile_backlog_tooling_exists(&standalone_tmp.path).is_err());
    Ok(())
}

#[test]
fn test_check_33_hardened_artifacts_and_substring_loophole_closed(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("c33_hardened")?;
    let pycache = tmp.path.join("__pycache__");
    fs::create_dir_all(&pycache)?;
    let req_txt = tmp.path.join("requirements.txt");
    File::create(&req_txt)?;
    let docs_dir = tmp.path.join("docs");
    fs::create_dir_all(&docs_dir)?;
    let doc_file = docs_dir.join("guide.md");
    let py_runner = join_pair("py", "test");
    let pip_cmd = join_pair("pip", " install");
    fs::write(
        &doc_file,
        format!(
            "Run {py_runner} to verify the banner never flickers.\nUse {pip_cmd} for deps.\n"
        ),
    )?;
    let res = check_zero_python_scripts(&tmp.path);
    assert!(res.is_err());
    let errs = res.err().ok_or("expected Err")?;
    assert!(errs.iter().any(|e| e.contains("__pycache__")));
    assert!(errs.iter().any(|e| e.contains("requirements.txt")));
    assert!(errs.iter().any(|e| e.contains("guide.md:1:")));
    assert!(errs.iter().any(|e| e.contains("guide.md:2:")));
    Ok(())
}

#[test]
fn test_audited_module_dirs_and_strict_io_arms() -> Result<(), Box<dyn std::error::Error>> {
    let overlap_probe = format!("let _ = Reg{}new();", join_pair("Reg", "ex::"));
    assert!(has_split_token(&overlap_probe, "Reg", "ex::"));
    assert!(has_split_token("prefix αβγδ suffix", "αβ", "γδ"));
    assert!(!has_split_token("let _ = RegOnly;", "Reg", "ex::"));
    assert!(!has_split_token("short", "longer_prefix", "x"));

    let cargo_probe = format!("{} = \"1.10\"", join_pair("reg", "ex"));
    let cargo_ident = format!("my_{}_helper = \"1.10\"", join_pair("reg", "ex"));
    assert!(contains_bounded_split_token(&cargo_probe, "reg", "ex"));
    assert!(!contains_bounded_split_token(&cargo_ident, "reg", "ex"));

    assert_eq!(AUDITED_MODULES.len(), 12);
    for spec in AUDITED_MODULES {
        let file_path = format!("{}.rs", spec.workspace_stem);
        let dir_sub_path = format!("{}/sub.rs", spec.workspace_stem);
        let sibling_path = format!("{}_extra.rs", spec.workspace_stem);
        assert!(is_audited_source_path(&file_path));
        assert!(is_audited_source_path(&dir_sub_path));
        assert!(!is_audited_source_path(&sibling_path));
        assert_eq!(
            enforces_strict_io_arms(&file_path),
            enforces_strict_io_arms(&dir_sub_path)
        );
    }
    assert!(!is_audited_source_path("crates/deap-core/src/lib.rs"));

    let tmp = TempDir::new("c34_decomposed_module_dirs")?;
    let markdown_sub_dir = tmp
        .path
        .join("crates")
        .join("deap-core")
        .join("src")
        .join("markdown");
    fs::create_dir_all(&markdown_sub_dir)?;
    fs::write(
        markdown_sub_dir.join("sub.rs"),
        "pub fn undocumented_in_decomposed_markdown_dir() {}\n",
    )?;
    assert!(check_zero_regex(&tmp.path).is_err());

    let tmp_purity = TempDir::new("c34_decomposed_compiler_purity")?;
    let valid_mod_src = clean_decomposed_tier_a_source();
    for spec in AUDITED_MODULES {
        let mod_dir = tmp_purity.path.join(spec.workspace_stem);
        fs::create_dir_all(&mod_dir)?;
        fs::write(mod_dir.join("mod.rs"), &valid_mod_src)?;
        fs::write(mod_dir.join("sub.rs"), &valid_mod_src)?;
    }
    assert!(check_compiler_ast_purity(&tmp_purity.path).is_ok());

    fs::write(
        tmp_purity
            .path
            .join("crates/ingest-sysml/src/translators/code/sub.rs"),
        "pub fn undocumented_sub() {}\n",
    )?;
    assert!(check_compiler_ast_purity(&tmp_purity.path).is_err());
    Ok(())
}

#[test]
fn test_strict_io_arms_and_self_scan_invariant() -> Result<(), Box<dyn std::error::Error>> {
    let err_arm = join_pair("Err(_)", " => continue");
    let io_loop_probe = format!(
        "pub fn parse_loop() {{ loop {{ match step() {{ Ok(()) => break, {err_arm} }} }} }}\n"
    );

    for exempt_path in [
        "crates/deap-core/src/sysml_ast.rs",
        "crates/deap-core/src/sysml_ast/sub.rs",
        "crates/verify-baseline/src/checks/parity_gates.rs",
        "crates/verify-baseline/src/checks/parity_gates/sub.rs",
    ] {
        assert!(audit_source_purity(&io_loop_probe, exempt_path, false).is_ok());
    }

    for strict_path in [
        "crates/verify-baseline/src/checks/structural.rs",
        "crates/verify-baseline/src/checks/structural/sub.rs",
        "crates/deap-core/src/workspace.rs",
        "crates/deap-core/src/workspace/sub.rs",
        "crates/deap-core/src/markdown.rs",
        "crates/deap-core/src/markdown/sub.rs",
        "crates/deap-core/src/source_ast.rs",
        "crates/deap-core/src/source_ast/sub.rs",
    ] {
        assert!(audit_source_purity(&io_loop_probe, strict_path, false).is_err());
    }

    let structural_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("checks")
        .join("structural.rs");
    let structural_text = fs::read_to_string(&structural_path)?;
    let forbidden_method = join_pair(".con", "cat(");
    let forbidden_macro = join_pair("con", "cat!");
    assert!(!structural_text.contains(&forbidden_method));
    assert!(!structural_text.contains(&forbidden_macro));
    Ok(())
}

/// Returns a minimal valid Tier A module source snippet for decomposed-directory purity tests.
fn clean_decomposed_tier_a_source() -> String {
    String::from(
        "//! Decomposed submodule fixture.\n\
         //!\n\
         //! Safety & Traceability: Realises: [REQ-DECOMPOSED-FIXTURE]\n\
         //! Contract: Pure deterministic test fixture.\n\
         //! Bounds & Faults: O(1) time and space; infallible.\n\n\
         /// [DOC-TIER: A] Evaluates deterministic constant.\n\
         ///\n\
         /// Safety & Traceability: Realises: [REQ-DECOMPOSED-FN]\n\
         /// Contract: Returns `1`.\n\
         /// Bounds & Faults: O(1) time and space; infallible.\n\
         pub const fn unit_value() -> usize {\n\
             1\n\
         }\n",
    )
}

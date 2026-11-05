use super::struct_check_a_tests::{join_pair, TempDir};
use super::structural::*;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

#[test]
fn test_issue_525_scan_clippy_allow_tokens_and_manifest_forbid_gate(
) -> Result<(), Box<dyn std::error::Error>> {
    let kw = join_pair("allow(clip", "py::");
    let clean_src = format!(
        "// #[{kw}unwrap_used)]\n\
         /* outer /* #[{kw}panic)] */ still comment */\n\
         pub fn demo() {{\n\
             let s = \"#[{kw}expect_used)]\";\n\
             let r = r#\"#![{kw}indexing_slicing)]\"#;\n\
             let ch = '#';\n\
             let br = '[';\n\
             let _pair = (s, r, ch, br);\n\
         }}\n"
    );
    let clean_viols = scan_clippy_allow_tokens(&clean_src, "crates/sample/src/lib.rs");
    assert!(clean_viols.is_empty());

    let dirty_src = format!(
        "#![{kw}indexing_slicing)]\n\
         #[{kw}unwrap_used)]\n\
         #[\n  allow(\n    clippy :: type_complexity\n  )\n]\n\
         #[cfg_attr(test, {kw}panic))]\n\
         pub fn bad() {{}}\n"
    );
    let dirty_viols = scan_clippy_allow_tokens(&dirty_src, "crates/sample/src/lib.rs");
    assert_eq!(dirty_viols.len(), 4);
    let v0 = dirty_viols.first().ok_or("missing v0")?;
    assert_eq!(v0.line, 1);
    assert_eq!(v0.attribute, format!("#![{kw}indexing_slicing)]"));
    let v1 = dirty_viols.get(1).ok_or("missing v1")?;
    assert_eq!(v1.line, 2);
    assert_eq!(v1.attribute, format!("#[{kw}unwrap_used)]"));
    let v2 = dirty_viols.get(2).ok_or("missing v2")?;
    assert_eq!(v2.line, 3);
    assert_eq!(v2.attribute, format!("#[{kw}type_complexity)]"));
    let v3 = dirty_viols.get(3).ok_or("missing v3")?;
    assert_eq!(v3.line, 8);
    assert_eq!(v3.attribute, format!("#[cfg_attr(test,{kw}panic))]"));

    let purity_res = audit_source_purity(&dirty_src, "crates/deap-core/src/rules.rs", false);
    let purity_errs = purity_res.err().ok_or("expected Err")?;
    assert!(purity_errs
        .iter()
        .any(|e| e.contains("forbidden clippy allow attribute detected")));
    Ok(())
}

#[test]
fn test_issue_525_manifest_forbid_gate_negative_and_live() -> Result<(), Box<dyn std::error::Error>>
{
    let tmp_bad = TempDir::new("c525_manifest_bad")?;
    {
        let mut f = File::create(tmp_bad.path.join("Cargo.toml"))?;
        writeln!(
            f,
            "[workspace]\nmembers = [\"crates/sample\"]\n\n\
             [workspace.lints.rust]\nunsafe_code = \"deny\"\n\n\
             [workspace.lints.clippy]\nunwrap_used = \"warn\"\n"
        )?;
    }
    let sample_dir = tmp_bad.path.join("crates").join("sample");
    fs::create_dir_all(&sample_dir)?;
    {
        let mut f = File::create(sample_dir.join("Cargo.toml"))?;
        writeln!(
            f,
            "[package]\nname = \"sample\"\nversion = \"0.1.0\"\n\
             [dependencies]\nclap = {{ version = \"4.4\", features = [\"derive\"] }}\n"
        )?;
    }
    let bad_res = WorkspaceManifestForbidGate::verify(&tmp_bad.path);
    let bad_errs = bad_res.err().ok_or("expected Err")?;
    assert!(bad_errs.iter().any(|e| e.contains("unsafe_code")));
    assert!(bad_errs.iter().any(|e| e.contains("unwrap_used")));
    assert!(bad_errs.iter().any(|e| e.contains("type_complexity")));
    assert!(bad_errs
        .iter()
        .any(|e| e.contains("missing [lints] workspace = true")));

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .ok_or("repo root")?;
    assert!(WorkspaceManifestForbidGate::verify(repo_root).is_ok());
    assert!(check_zero_regex(repo_root).is_ok());
    Ok(())
}

#[test]
fn test_issue_526_self_hosting_purity_and_fail_closed_gates(
) -> Result<(), Box<dyn std::error::Error>> {
    let chained_mutant = "let sub = &arr[0][1..2];";
    let res_chained =
        audit_source_purity(chained_mutant, "sample/src/checks/parity_gates.rs", true);
    assert!(res_chained
        .err()
        .ok_or("expected Err")?
        .iter()
        .any(|e| e.contains("direct panicking range-slice indexing detected")));

    let hollow = join_pair("let", " _ = ");
    let mod_str = join_pair("mod", " tests");
    let literal_bypass_mutant =
        format!("pub fn prod(x: i32) {{\n    let s = \"{mod_str}\";\n    {hollow}x;\n}}\n");
    assert!(audit_source_purity(&literal_bypass_mutant, "sample/src/lib.rs", false).is_err());

    let closed_mod_bypass_mutant = format!(
        "mod tests {{\n    #[test]\n    fn valid_test() {{ assert_eq!(1, 1); }}\n}}\npub fn sneaky(x: i32) {{\n    {hollow}x;\n}}\n"
    );
    assert!(audit_source_purity(&closed_mod_bypass_mutant, "sample/src/lib.rs", false).is_err());

    let ign = join_pair("#[ig", "nore]");
    assert!(audit_source_purity(
        &format!("mod tests {{\n    #[test]\n    {ign}\n    fn t() {{ assert_eq!(1, 1); }}\n}}\n"),
        "sample/src/checks/parity_gates.rs",
        true,
    )
    .is_err());

    let taut = join_pair("assert!", "(true)");
    assert!(audit_source_purity(
        &format!("mod tests {{\n    #[test]\n    fn t() {{ {taut}; }}\n}}\n"),
        "sample/src/checks/parity_gates.rs",
        true,
    )
    .is_err());

    let vacuous = join_pair("if p.ex", "ists() { return; }");
    assert!(audit_source_purity(
        &format!("mod tests {{\n    #[test]\n    fn t() {{ {vacuous} }}\n}}\n"),
        "sample/src/checks/parity_gates.rs",
        true,
    )
    .is_err());

    let unisolated = join_pair("temp_dir()", ".join(\"fixed_shared_dir\")");
    assert!(audit_source_purity(
        &format!("mod tests {{\n    #[test]\n    fn t() {{ let d = std::env::{unisolated}; assert!(d.is_Absolute() || true); }}\n}}\n"),
        "sample/src/checks/parity_gates.rs",
        true,
    )
    .is_err());

    let decoy_mod = join_pair("mod", " tests {}");
    assert!(audit_source_purity(&decoy_mod, "sample/src/checks/parity_gates.rs", true).is_err());
    Ok(())
}

#[test]
fn test_issue_526_rule_d_and_missing_audited_files() -> Result<(), Box<dyn std::error::Error>> {
    let err_cont = join_pair("Err(_)", " => continue");
    assert!(audit_source_purity(
        &format!("pub fn scan() {{ match res {{ Ok(v) => v, {err_cont} }} }}\n"),
        "sample/src/lib.rs",
        false,
    )
    .is_err());

    let if_ok_read = join_pair("if let ", "Ok(c) = fs::read_to_string(p)");
    assert!(audit_source_purity(
        &format!("pub fn scan() {{ {if_ok_read} {{}} }}\n"),
        "sample/src/lib.rs",
        false,
    )
    .is_err());

    let tmp = TempDir::new("c526_missing_audited")?;
    assert!(check_compiler_ast_purity(&tmp.path).is_err());
    Ok(())
}

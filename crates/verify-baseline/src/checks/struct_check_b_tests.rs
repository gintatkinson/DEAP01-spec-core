use super::struct_check_a_tests::{join_pair, TempDir};
use super::structural::*;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

#[test]
fn test_check_32_zero_shell_scripts_exempts_build_dirs_and_rejects_others(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("c32_exempt_dirs")?;
    for dir in [
        "build/macos",
        "macos/Pods",
        "macos/Flutter/ephemeral",
        ".dart_tool/build",
    ] {
        let d = tmp.path.join(dir);
        fs::create_dir_all(&d)?;
        File::create(d.join("generated.sh"))?;
    }
    assert!(
        check_zero_shell_scripts(&tmp.path).is_ok(),
        "Generated .sh files inside build, Pods, ephemeral, and .dart_tool must be exempted"
    );

    let forbidden_sh = tmp.path.join("scripts").join("deploy.sh");
    let parent_dir = forbidden_sh.parent().ok_or("missing parent")?;
    fs::create_dir_all(parent_dir)?;
    File::create(&forbidden_sh)?;
    let res = check_zero_shell_scripts(&tmp.path);
    assert!(res.is_err(), "Non-exempt .sh script must fail Check 32");
    let errs = res.err().ok_or("expected Err")?;
    assert_eq!(errs, vec!["scripts/deploy.sh".to_string()]);
    Ok(())
}

#[test]
fn test_check_33_zero_python_clean() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("c33_clean")?;
    let rs_file = tmp.path.join("main.rs");
    File::create(&rs_file)?;
    assert!(check_zero_python_scripts(&tmp.path).is_ok());
    Ok(())
}

#[test]
fn test_check_33_zero_python_exempts_build_dirs_and_rejects_others(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("c33_exempt_dirs")?;
    for dir in [
        "build/macos",
        "macos/Pods",
        "macos/Flutter/ephemeral",
        ".dart_tool/build",
    ] {
        let d = tmp.path.join(dir);
        fs::create_dir_all(&d)?;
        File::create(d.join("helper.py"))?;
    }
    assert!(
        check_zero_python_scripts(&tmp.path).is_ok(),
        "Generated .py files inside build, Pods, ephemeral, and .dart_tool must be exempted"
    );

    let forbidden_py = tmp.path.join("scripts").join("legacy.py");
    let parent_dir = forbidden_py.parent().ok_or("missing parent")?;
    fs::create_dir_all(parent_dir)?;
    File::create(&forbidden_py)?;
    let res = check_zero_python_scripts(&tmp.path);
    assert!(res.is_err(), "Non-exempt .py script must fail Check 33");
    let errs = res.err().ok_or("expected Err")?;
    assert_eq!(errs, vec!["scripts/legacy.py".to_string()]);
    Ok(())
}

#[test]
fn test_check_33_zero_python_violation() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("c33_violation")?;
    let py_file = tmp.path.join("script.py");
    File::create(&py_file)?;
    let res = check_zero_python_scripts(&tmp.path);
    assert!(res.is_err());
    let errs = res.err().ok_or("expected Err")?;
    assert_eq!(errs.len(), 1);
    assert_eq!(errs.first().map(String::as_str), Some("script.py"));
    Ok(())
}

#[test]
fn test_check_34_zero_regex_clean() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("c34_clean")?;
    let crates_dir = tmp.path.join("crates").join("sample").join("src");
    fs::create_dir_all(&crates_dir)?;
    let rs_file = crates_dir.join("lib.rs");
    let mut f = File::create(&rs_file)?;
    writeln!(f, "pub fn add(a: i32, b: i32) -> i32 {{ a + b }}")?;

    let cargo_file = tmp.path.join("crates").join("sample").join("Cargo.toml");
    let mut fc = File::create(&cargo_file)?;
    writeln!(fc, "[package]\nname = \"sample\"\nversion = \"0.1.0\"")?;

    assert!(check_zero_regex(&tmp.path).is_ok());
    Ok(())
}

#[test]
fn test_check_34_zero_regex_violation_rs() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("c34_viol_rs")?;
    let crates_dir = tmp.path.join("crates").join("sample").join("src");
    fs::create_dir_all(&crates_dir)?;
    let rs_file = crates_dir.join("lib.rs");
    let mut f = File::create(&rs_file)?;
    let pat = join_pair("Reg", "ex::");
    writeln!(f, "let _ = {}new(\"\");", pat)?;

    let res = check_zero_regex(&tmp.path);
    assert!(res.is_err());
    let errs = res.err().ok_or("expected Err")?;
    assert_eq!(errs.len(), 1);
    assert!(errs
        .first()
        .is_some_and(|e| e.contains("crates/sample/src/lib.rs:1:")));
    Ok(())
}

#[test]
fn test_check_34_zero_regex_violation_cargo() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("c34_viol_cargo")?;
    let crates_dir = tmp.path.join("crates").join("sample");
    fs::create_dir_all(&crates_dir)?;
    let cargo_file = crates_dir.join("Cargo.toml");
    let mut fc = File::create(&cargo_file)?;
    let dep = join_pair("reg", "ex");
    writeln!(fc, "[dependencies]\n{} = \"1.10\"", dep)?;

    let res = check_zero_regex(&tmp.path);
    assert!(res.is_err());
    let errs = res.err().ok_or("expected Err")?;
    assert_eq!(errs.len(), 1);
    assert!(errs
        .first()
        .is_some_and(|e| e.contains("crates/sample/Cargo.toml:2:")));
    Ok(())
}

#[test]
fn test_check_34_repo_root_passes() -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .ok_or("repo root")?;
    let res = check_zero_regex(repo_root);
    assert!(
        res.is_ok(),
        "Check 34 Zero Regex Invariant failed on repository root: {:?}",
        res.err()
    );
    Ok(())
}

#[test]
fn test_check_34_audited_parity_gates_passes() -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .ok_or("repo root")?;
    let res = check_compiler_ast_purity(repo_root);
    assert!(
        res.is_ok(),
        "Check 34 Compiler AST Purity failed on audited modules: {:?}",
        res.err()
    );

    let structural_path = repo_root
        .join("crates")
        .join("verify-baseline")
        .join("src")
        .join("checks")
        .join("structural.rs");
    let structural_src = fs::read_to_string(&structural_path)?;
    let res_structural = audit_source_purity(
        &structural_src,
        "crates/verify-baseline/src/checks/structural.rs",
        true,
    );
    assert!(
        res_structural.is_ok(),
        "structural.rs failed audit_source_purity: {:?}",
        res_structural.err()
    );
    Ok(())
}

#[test]
fn test_check_34_purity_a_rejects_regex() -> Result<(), Box<dyn std::error::Error>> {
    let clean = "pub fn add(a: i32, b: i32) -> i32 { a + b }";
    let res_clean = audit_source_purity(clean, "sample/src/lib.rs", false);
    assert!(
        res_clean.is_ok(),
        "Clean source must pass: {:?}",
        res_clean.err()
    );

    let pat = join_pair("Reg", "ex::");
    let mutant = format!("pub fn check() {{ {}new(\"\"); }}", pat);
    let res_mutant = audit_source_purity(&mutant, "sample/src/lib.rs", false);
    assert!(res_mutant.is_err(), "Mutant with regex must fail");
    let errs = res_mutant.err().ok_or("expected Err")?;
    assert!(errs.iter().any(|e| e.contains("regex usage detected")));
    Ok(())
}

#[test]
fn test_check_34_purity_b_rejects_static_string_slices() -> Result<(), Box<dyn std::error::Error>> {
    let clean = "pub const ALL: &'static [Self] = &[Self::VariantA, Self::VariantB];";
    let res_clean = audit_source_purity(clean, "sample/src/lib.rs", false);
    assert!(
        res_clean.is_ok(),
        "Clean enum array must pass: {:?}",
        res_clean.err()
    );

    let slice_kw = join_pair(": &[", "&str]");
    let mutant = format!("pub const KEYWORDS{kw} = &[\"forbidden\"];", kw = slice_kw);
    let res_mutant = audit_source_purity(&mutant, "sample/src/lib.rs", false);
    assert!(
        res_mutant.is_err(),
        "Mutant with static string slice array must fail"
    );
    let errs = res_mutant.err().ok_or("expected Err")?;
    assert!(errs
        .iter()
        .any(|e| e.contains("static string-slice array detected")));
    Ok(())
}

#[test]
fn test_check_34_purity_c_rejects_hollow_gate_stubs() -> Result<(), Box<dyn std::error::Error>> {
    let clean = "pub fn check_gate(repo_root: &Path) -> Result<(), Vec<String>> { if repo_root.is_dir() { Ok(()) } else { Err(vec![]) } }";
    let res_clean = audit_source_purity(clean, "sample/src/checks/gate.rs", false);
    assert!(
        res_clean.is_ok(),
        "Clean genuine gate must pass: {:?}",
        res_clean.err()
    );

    let hollow_pat = join_pair("let", " _ = ");
    let mutant = format!(
        "pub fn check_gate(repo_root: &Path) -> Result<(), Vec<String>> {{ {}repo_root; Ok(()) }}",
        hollow_pat
    );
    let res_mutant = audit_source_purity(&mutant, "sample/src/checks/gate.rs", false);
    assert!(
        res_mutant.is_err(),
        "Mutant with hollow gate stub must fail"
    );
    let errs = res_mutant.err().ok_or("expected Err")?;
    assert!(errs.iter().any(|e| e.contains("hollow gate stub detected")));
    Ok(())
}

#[test]
fn test_check_34_purity_d_rejects_fail_open_error_swallowing(
) -> Result<(), Box<dyn std::error::Error>> {
    let clean = "for entry_res in WalkDir::new(dir) { let entry = match entry_res { Ok(e) => e, Err(err) => return Err(vec![err.to_string()]) }; }";
    let res_clean = audit_source_purity(clean, "sample/src/checks/gate.rs", false);
    assert!(
        res_clean.is_ok(),
        "Clean fail-closed loop must pass: {:?}",
        res_clean.err()
    );

    let fm_pat = join_pair(".filter_map(|e|", " e.ok())");
    let mutant = format!("for entry in WalkDir::new(dir).into_iter(){} {{ }}", fm_pat);
    let res_mutant = audit_source_purity(&mutant, "sample/src/checks/gate.rs", false);
    assert!(
        res_mutant.is_err(),
        "Mutant with fail-open filter_map must fail"
    );
    let errs = res_mutant.err().ok_or("expected Err")?;
    assert!(errs
        .iter()
        .any(|e| e.contains("fail-open error swallowing detected")));
    Ok(())
}

#[test]
fn test_check_34_purity_e_rejects_cross_string_byte_slicing(
) -> Result<(), Box<dyn std::error::Error>> {
    let clean = "let lower = s.to_ascii_lowercase(); let slice = lower.get(..4);";
    let res_clean = audit_source_purity(clean, "sample/src/lib.rs", false);
    assert!(
        res_clean.is_ok(),
        "Clean character boundary slicing must pass: {:?}",
        res_clean.err()
    );

    let bad_slice = join_pair(".to_lowercase()", "[..4]");
    let mutant = format!("let slice = &s{};", bad_slice);
    let res_mutant = audit_source_purity(&mutant, "sample/src/lib.rs", false);
    assert!(
        res_mutant.is_err(),
        "Mutant with cross-string byte indexing must fail"
    );
    let errs = res_mutant.err().ok_or("expected Err")?;
    assert!(errs
        .iter()
        .any(|e| e.contains("cross-string UTF-8 byte-slice hazard detected")));
    Ok(())
}

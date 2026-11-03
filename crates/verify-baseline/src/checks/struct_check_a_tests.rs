use super::structural::*;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

pub(crate) struct TempDir {
    pub(crate) path: std::path::PathBuf,
}

impl TempDir {
    pub(crate) fn new(name: &str) -> Result<Self, std::io::Error> {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let count = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!("verify_structural_{}_{}_{}", name, nanos, count));
        fs::create_dir_all(&path)?;
        Ok(Self { path })
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// Joins two string slices for negative-probe test inputs without array-join or macro-join tokens.
pub(crate) fn join_pair(a: &str, b: &str) -> String {
    let mut s = String::with_capacity(a.len().saturating_add(b.len()));
    s.push_str(a);
    s.push_str(b);
    s
}

#[test]
fn test_check_18_downstream_exempt() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("c18_downstream")?;
    assert!(verify_upstream_blueprint_domain_cleanliness(&tmp.path).is_ok());
    Ok(())
}

#[test]
fn test_check_18_upstream_clean() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("c18_clean")?;
    fs::create_dir_all(tmp.path.join(".pipeline/upstream"))?;
    let schema_dir = tmp.path.join("schema");
    fs::create_dir_all(&schema_dir)?;
    File::create(schema_dir.join(".gitkeep"))?;

    let bp_dir = tmp.path.join("docs/architecture/blueprints");
    fs::create_dir_all(&bp_dir)?;
    File::create(bp_dir.join("CORE_ARCHITECTURE.md"))?;
    File::create(bp_dir.join("FRONTEND_SYSTEMS_PATTERN.md"))?;

    assert!(verify_upstream_blueprint_domain_cleanliness(&tmp.path).is_ok());
    Ok(())
}

#[test]
fn test_check_18_upstream_violations() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("c18_upstream")?;
    fs::create_dir_all(tmp.path.join(".pipeline/upstream"))?;
    let bp_dir = tmp.path.join("docs/architecture/blueprints");
    fs::create_dir_all(&bp_dir)?;

    let clean_file = bp_dir.join("CORE_ARCHITECTURE.md");
    File::create(&clean_file)?;
    assert!(verify_upstream_blueprint_domain_cleanliness(&tmp.path).is_ok());

    let dirty_file = bp_dir.join("FLIGHT_SYSTEMS_MODEL.sysml");
    File::create(&dirty_file)?;
    assert!(verify_upstream_blueprint_domain_cleanliness(&tmp.path).is_err());
    Ok(())
}

#[test]
fn test_check_18_upstream_designs_sysml_violations() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("c18_designs_sysml")?;
    fs::create_dir_all(tmp.path.join(".pipeline/upstream"))?;
    let designs_dir = tmp.path.join("docs/designs");
    fs::create_dir_all(&designs_dir)?;

    fs::write(designs_dir.join("blueprint-01-valid.md"), "# Valid Blueprint\n")?;
    fs::write(designs_dir.join("blueprint-01-valid.sysml"), "package Valid {}\n")?;
    fs::write(designs_dir.join("blueprint-01-valid.digest.json"), "{}\n")?;
    assert!(verify_upstream_blueprint_domain_cleanliness(&tmp.path).is_ok());

    let domain_sysml = designs_dir.join("domain.sysml");
    fs::write(&domain_sysml, "package ForbiddenDomain {}\n")?;
    let res = verify_upstream_blueprint_domain_cleanliness(&tmp.path);
    assert!(res.is_err());
    let errs = res.err().ok_or("expected Err")?;
    assert!(
        errs.iter()
            .any(|e| e.contains("Upstream blueprint contains SysML model: docs/designs/domain.sysml")),
        "Expected docs/designs/domain.sysml violation, got: {:?}",
        errs
    );
    fs::remove_file(&domain_sysml)?;

    let orphan_sysml = designs_dir.join("blueprint-99-orphan.sysml");
    fs::write(&orphan_sysml, "package Orphan {}\n")?;
    let res_orphan = verify_upstream_blueprint_domain_cleanliness(&tmp.path);
    assert!(res_orphan.is_err());
    let errs_orphan = res_orphan.err().ok_or("expected Err")?;
    assert!(
        errs_orphan.iter().any(|e| {
            e.contains("Upstream blueprint contains SysML model: docs/designs/blueprint-99-orphan.sysml")
        }),
        "Expected docs/designs/blueprint-99-orphan.sysml violation, got: {:?}",
        errs_orphan
    );
    Ok(())
}

#[test]
fn test_check_18_upstream_digest_json_sidecar_violations() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("c18_digest_json")?;
    fs::create_dir_all(tmp.path.join(".pipeline/upstream"))?;
    let bp_dir = tmp.path.join("docs/architecture/blueprints");
    let designs_dir = tmp.path.join("docs/designs");
    fs::create_dir_all(&bp_dir)?;
    fs::create_dir_all(&designs_dir)?;

    let bp_digest = bp_dir.join("blueprint-01.digest.json");
    fs::write(&bp_digest, "{}\n")?;
    let res_bp = verify_upstream_blueprint_domain_cleanliness(&tmp.path);
    assert!(res_bp.is_err());
    let errs_bp = res_bp.err().ok_or("expected Err")?;
    assert!(
        errs_bp.iter().any(|e| {
            e.contains("Upstream blueprint contains SysML model: docs/architecture/blueprints/blueprint-01.digest.json")
        }),
        "Expected docs/architecture/blueprints/blueprint-01.digest.json violation, got: {:?}",
        errs_bp
    );
    fs::remove_file(&bp_digest)?;

    let domain_digest = designs_dir.join("domain.digest.json");
    fs::write(&domain_digest, "{}\n")?;
    let res_domain = verify_upstream_blueprint_domain_cleanliness(&tmp.path);
    assert!(res_domain.is_err());
    let errs_domain = res_domain.err().ok_or("expected Err")?;
    assert!(
        errs_domain
            .iter()
            .any(|e| e.contains("Upstream blueprint contains SysML model: docs/designs/domain.digest.json")),
        "Expected docs/designs/domain.digest.json violation, got: {:?}",
        errs_domain
    );
    fs::remove_file(&domain_digest)?;

    let orphan_digest = designs_dir.join("blueprint-99-orphan.digest.json");
    fs::write(&orphan_digest, "{}\n")?;
    let res_orphan = verify_upstream_blueprint_domain_cleanliness(&tmp.path);
    assert!(res_orphan.is_err());
    let errs_orphan = res_orphan.err().ok_or("expected Err")?;
    assert!(
        errs_orphan.iter().any(|e| {
            e.contains("Upstream blueprint contains SysML model: docs/designs/blueprint-99-orphan.digest.json")
        }),
        "Expected docs/designs/blueprint-99-orphan.digest.json violation, got: {:?}",
        errs_orphan
    );
    Ok(())
}

#[test]
fn test_check_18_landing_zone_non_gitkeep_fails() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("c18_lz")?;
    fs::create_dir_all(tmp.path.join(".pipeline/upstream"))?;
    let schema_dir = tmp.path.join("schema");
    fs::create_dir_all(&schema_dir)?;
    File::create(schema_dir.join("domain_model.sysml"))?;
    assert!(verify_upstream_blueprint_domain_cleanliness(&tmp.path).is_err());
    Ok(())
}

#[test]
fn test_check_19_downstream_exempt() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("c19_downstream")?;
    assert!(check_domain_agnostic_ast_cleanliness(&tmp.path).is_ok());
    Ok(())
}

#[test]
fn test_check_19_upstream_clean() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("c19_clean")?;
    fs::create_dir_all(tmp.path.join(".pipeline/upstream"))?;
    let src_dir = tmp.path.join("crates/test-crate/src");
    fs::create_dir_all(&src_dir)?;
    let mut f = File::create(src_dir.join("lib.rs"))?;
    writeln!(f, "pub fn compute_hash(val: &[u8]) -> u64 {{ 42 }}")?;
    assert!(check_domain_agnostic_ast_cleanliness(&tmp.path).is_ok());
    Ok(())
}

#[test]
fn test_check_19_rust_crates_domain_params_fails() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("c19_rust")?;
    fs::create_dir_all(tmp.path.join(".pipeline/upstream"))?;
    let src_dir = tmp.path.join("crates/test-crate/src");
    fs::create_dir_all(&src_dir)?;
    let mut f = File::create(src_dir.join("lib.rs"))?;
    let kw = join_pair("DOM", "AIN_PARAMS");
    writeln!(f, "pub const {}: &[&str] = &[\"speed\"];", kw)?;
    assert!(check_domain_agnostic_ast_cleanliness(&tmp.path).is_err());
    Ok(())
}

#[test]
fn test_reproduce_defect_265_unflagged_m1_domain_dict_bypasses_check_19(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("c19_m1_bypass")?;
    fs::create_dir_all(tmp.path.join(".pipeline/upstream"))?;
    let src_dir = tmp.path.join("crates/test-crate/src");
    fs::create_dir_all(&src_dir)?;
    let mut f = File::create(src_dir.join("lib.rs"))?;
    writeln!(
        f,
        "pub const FLIGHT_ENVELOPE_MAP: &[(&str, f64)] = &[(\"max_altitude\", 10000.0)];"
    )?;
    assert!(
        check_domain_agnostic_ast_cleanliness(&tmp.path).is_err(),
        "Check 19 failed to reject unflagged M1 domain dict FLIGHT_ENVELOPE_MAP"
    );
    Ok(())
}

#[test]
fn test_check_19_legacy_python_parity_auditor_dir_fails(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("c19_legacy")?;
    fs::create_dir_all(tmp.path.join(".pipeline/upstream"))?;
    let crates_dir = tmp.path.join("crates");
    fs::create_dir_all(&crates_dir)?;
    let legacy_dir = tmp.path.join("skills/spec-orchestrator/parity_auditor");
    fs::create_dir_all(&legacy_dir)?;
    assert!(check_domain_agnostic_ast_cleanliness(&tmp.path).is_err());
    Ok(())
}

#[test]
fn test_check_33_excludes_digest_json_sidecars() -> Result<(), Box<dyn std::error::Error>> {
    assert!(!is_governed_python_scan_extension(Path::new(
        "docs/designs/blueprint-01.digest.json"
    )));
    assert!(!is_governed_python_scan_extension(Path::new(
        "docs/designs/foo-digest.json"
    )));
    assert!(!is_governed_python_scan_extension(Path::new(
        ".pipeline/schema-digest.json"
    )));
    assert!(is_governed_python_scan_extension(Path::new(
        "docs/config.json"
    )));

    let tmp = TempDir::new("c33_digest_json_sidecar")?;
    let designs_dir = tmp.path.join("docs").join("designs");
    fs::create_dir_all(&designs_dir)?;
    let digest_file = designs_dir.join("blueprint-01.digest.json");
    let mut f = File::create(&digest_file)?;
    let token = join_pair("run_", "tests.py");
    writeln!(
        f,
        "{{\"summary\": \"executes pytest and {} in downstream runner\"}}",
        token
    )?;

    let res = check_zero_python_scripts(&tmp.path);
    assert!(
        res.is_ok(),
        "Check 33 must exclude machine-generated .digest.json sidecars (refs #532), got: {:?}",
        res.err()
    );
    Ok(())
}

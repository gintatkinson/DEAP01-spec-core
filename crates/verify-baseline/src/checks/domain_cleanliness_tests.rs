#![cfg(test)]
//! Unit tests for Check 19 (`domain_cleanliness_gate`).
//! Realises: [REQ-CHECKS-DOMAIN-CLEANLINESS-GATE-CHECK-DOMAIN-AGNOSTIC-AST-CLEANLINESS]

use super::*;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

struct TempDir {
    path: std::path::PathBuf,
}

impl TempDir {
    fn new(name: &str) -> Result<Self, std::io::Error> {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let count = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path =
            std::env::temp_dir().join(format!("verify_domain_{}_{}_{}", name, nanos, count));
        fs::create_dir_all(&path)?;
        Ok(Self { path })
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn test_is_allowed_m2_type() {
    assert!(is_allowed_m2_type("Component"));
    assert!(is_allowed_m2_type("Class"));
    assert!(is_allowed_m2_type("Port"));
    assert!(is_allowed_m2_type("PackageDefinition"));
    assert!(is_allowed_m2_type("HumanOperator"));
    assert!(is_allowed_m2_type("meta_custom_node"));
    assert!(is_allowed_m2_type("MetaPart"));
    assert!(is_allowed_m2_type("component"));
    assert!(is_allowed_m2_type("part_definition"));

    assert!(!is_allowed_m2_type("TacticalUAV"));
    assert!(!is_allowed_m2_type("FixedWingDrone"));
    assert!(!is_allowed_m2_type("FlightVehicle"));
    assert!(!is_allowed_m2_type("ConcreteActuatorMotor"));
    assert!(!is_allowed_m2_type("HospitalInfusionPump"));
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
fn test_check_19_rejects_unflagged_m1_domain_dict() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("c19_m1_reject")?;
    fs::create_dir_all(tmp.path.join(".pipeline/upstream"))?;
    let src_dir = tmp.path.join("crates/test-crate/src");
    fs::create_dir_all(&src_dir)?;
    let mut f = File::create(src_dir.join("lib.rs"))?;
    writeln!(
        f,
        "pub const FLIGHT_ENVELOPE_MAP: &[(&str, f64)] = &[(\"max_altitude\", 10000.0)];"
    )?;
    let res = check_domain_agnostic_ast_cleanliness(&tmp.path);
    assert!(res.is_err());
    let errs = res.err().ok_or("expected Err")?;
    assert!(errs
        .iter()
        .any(|e| e.contains("domain-metamodel-typing-violation")
            && e.contains("FLIGHT_ENVELOPE_MAP")));
    Ok(())
}

#[test]
fn test_check_19_rejects_unvalidated_m1_entity_type() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("c19_m1_type")?;
    fs::create_dir_all(tmp.path.join(".pipeline/upstream"))?;
    let src_dir = tmp.path.join("crates/test-crate/src");
    fs::create_dir_all(&src_dir)?;
    let mut f = File::create(src_dir.join("lib.rs"))?;
    writeln!(f, "let config = (\"metamodel_type\", \"TacticalUAV\");")?;
    let res = check_domain_agnostic_ast_cleanliness(&tmp.path);
    assert!(res.is_err());
    let errs = res.err().ok_or("expected Err")?;
    assert!(errs
        .iter()
        .any(|e| e.contains("domain-metamodel-typing-violation") && e.contains("TacticalUAV")));
    Ok(())
}

#[test]
fn test_check_19_accepts_valid_m2_metamodel_type() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("c19_m2_valid")?;
    fs::create_dir_all(tmp.path.join(".pipeline/upstream"))?;
    let src_dir = tmp.path.join("crates/test-crate/src");
    fs::create_dir_all(&src_dir)?;
    let mut f = File::create(src_dir.join("lib.rs"))?;
    writeln!(f, "let config = (\"metamodel_type\", \"Component\");")?;
    assert!(check_domain_agnostic_ast_cleanliness(&tmp.path).is_ok());
    Ok(())
}

#[test]
fn test_check_19_template_rejection_of_unmapped_domain_dict(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("c19_tmpl_reject")?;
    fs::create_dir_all(tmp.path.join(".pipeline/upstream"))?;
    fs::create_dir_all(tmp.path.join("crates"))?;
    let tmpl_dir = tmp
        .path
        .join("skills/spec-conops-engineering/resources/units");
    fs::create_dir_all(&tmpl_dir)?;
    let mut f = File::create(tmpl_dir.join("01_TEST_UNIT.md"))?;
    writeln!(f, "# Test Unit\n\nHardcoded payload: FLIGHT_ENVELOPE_MAP")?;
    let res = check_domain_agnostic_ast_cleanliness(&tmp.path);
    assert!(res.is_err());
    let errs = res.err().ok_or("expected Err")?;
    assert!(errs
        .iter()
        .any(|e| e.contains("domain-metamodel-typing-violation")
            && e.contains("FLIGHT_ENVELOPE_MAP")));
    Ok(())
}

#[test]
fn test_check_19_template_em_dash_rejection() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("c19_tmpl_emdash")?;
    fs::create_dir_all(tmp.path.join(".pipeline/upstream"))?;
    fs::create_dir_all(tmp.path.join("crates"))?;
    let tmpl_dir = tmp.path.join(".pipeline/templates");
    fs::create_dir_all(&tmpl_dir)?;
    let mut f = File::create(tmpl_dir.join("spec.yaml"))?;
    writeln!(f, "description: Bad em dash \u{2014} here")?;
    let res = check_domain_agnostic_ast_cleanliness(&tmp.path);
    assert!(res.is_err());
    let errs = res.err().ok_or("expected Err")?;
    assert!(errs.iter().any(|e| e.contains("Unicode em dash")));
    Ok(())
}

#[test]
fn test_check_19_rule_empennage_rejection() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("c19_rule_empennage")?;
    fs::create_dir_all(tmp.path.join(".pipeline/upstream"))?;
    fs::create_dir_all(tmp.path.join("crates"))?;
    let rules_dir = tmp.path.join("rules");
    fs::create_dir_all(&rules_dir)?;
    let mut f = File::create(rules_dir.join("test-rule.md"))?;
    writeln!(f, "# Rule\n\nControl surface: cruciform empennage")?;
    let res = check_domain_agnostic_ast_cleanliness(&tmp.path);
    assert!(res.is_err());
    let errs = res.err().ok_or("expected Err")?;
    assert!(errs.iter().any(|e| e.contains("empennage")));
    Ok(())
}

#[test]
fn test_check_19_conops_governance_rule_aerospace_decontamination(
) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .ok_or("repo root")?;
    let rule_path = repo_root.join("rules/conops-mission-intent-integrity.md");
    assert!(
        rule_path.is_file(),
        "rules/conops-mission-intent-integrity.md must exist"
    );
    let content = fs::read_to_string(&rule_path)?;

    let forbidden_terms = [
        "civil airborne",
        "bird strike",
        "mid-air",
        "airframe limits",
        "pitot probe",
        "propeller delamination",
        "pilot fatigue",
        "DO-178C",
        "ARP4761",
        "ARP4754A",
        "h_max",
    ];

    let mut violations = Vec::new();
    for term in &forbidden_terms {
        if content.to_lowercase().contains(&term.to_lowercase()) {
            violations.push(*term);
        }
    }
    assert!(
        violations.is_empty(),
        "rules/conops-mission-intent-integrity.md contains forbidden aerospace domain concepts: {:?}",
        violations
    );
    Ok(())
}

#[test]
fn test_check_19_current_repo_passes() -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .ok_or("repo root")?;
    let res = check_domain_agnostic_ast_cleanliness(repo_root);
    assert!(res.is_ok(), "Current repo failed Check 19: {:?}", res.err());
    Ok(())
}

#[test]
fn test_check_19_fails_closed_on_unreadable_non_utf8_files(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("c19_fail_closed_io")?;
    fs::create_dir_all(tmp.path.join(".pipeline/upstream"))?;
    let src_dir = tmp.path.join("crates/test-crate/src");
    fs::create_dir_all(&src_dir)?;
    fs::write(src_dir.join("lib.rs"), [0xFF, 0xFE, 0x00, 0x80])?;

    let rules_dir = tmp.path.join("rules");
    fs::create_dir_all(&rules_dir)?;
    fs::write(rules_dir.join("corrupt.md"), [0xFF, 0xFE, 0xFD])?;

    let res = check_domain_agnostic_ast_cleanliness(&tmp.path);
    assert!(
        res.is_err(),
        "Check 19 must fail closed on non-UTF-8 Rust or rule files, but returned Ok(())"
    );
    let errs = res.err().ok_or("expected Err")?;
    assert!(errs.iter().any(|e| e.contains("lib.rs")));
    assert!(errs.iter().any(|e| e.contains("corrupt.md")));
    Ok(())
}

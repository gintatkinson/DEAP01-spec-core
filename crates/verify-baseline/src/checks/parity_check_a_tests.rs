//! Unit tests (part A) for `parity_gates`: Check 31 dual-schema parity, Check 28 schema contract, and Check 27 SV-1/ICD parity.
//!
//! Safety & Traceability: Realises: [REQ-DUAL-SCHEMA-SSOT-PARITY], [REQ-PARITY-GATE-27], [REQ-PARITY-GATE-28]
//! Contract: Exercises dual-schema parity, ConOps schema validation, and cross-document SV-1/ICD connection parity.
//! Bounds & Faults: Uses auto-cleaned temporary directories (`TempDir`); zero shared mutable state.

use super::parity_gates::*;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

pub(crate) struct TempDir {
    pub(crate) path: PathBuf,
}

impl TempDir {
    pub(crate) fn new(name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let count = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("verify_parity_{}_{}_{}", name, nanos, count));
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
fn test_dual_schema_single_passes() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("single")?;
    assert_eq!(
        check_dual_schema_ssot_parity(&tmp.path),
        Ok(ParityOutcome::DualSchemaSingleOrClean)
    );
    Ok(())
}

#[test]
fn test_dual_schema_identical_passes() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("identical")?;
    let schema_dir = tmp.path.join("schema");
    let pipe_dir = tmp.path.join(".pipeline");
    fs::create_dir_all(&schema_dir)?;
    fs::create_dir_all(&pipe_dir)?;

    let sysml =
        "package Core {\n  part def Sensor;\n  port def TelemetryPort;\n  action def ReadSensor;\n}\n";
    let mut f1 = File::create(schema_dir.join("domain.sysml"))?;
    write!(f1, "{}", sysml)?;

    let mut f2 = File::create(pipe_dir.join("schema.sysml"))?;
    write!(f2, "{}", sysml)?;

    assert_eq!(
        check_dual_schema_ssot_parity(&tmp.path),
        Ok(ParityOutcome::DualSchemaIdentical)
    );
    Ok(())
}

#[test]
fn test_dual_schema_mismatch_fails() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("mismatch")?;
    let schema_dir = tmp.path.join("schema");
    let pipe_dir = tmp.path.join(".pipeline");
    fs::create_dir_all(&schema_dir)?;
    fs::create_dir_all(&pipe_dir)?;

    let sysml1 = "package Core {\n  part def Sensor;\n}\n";
    let mut f1 = File::create(schema_dir.join("domain.sysml"))?;
    write!(f1, "{}", sysml1)?;

    let sysml2 = "package Core {\n  part def Actuator;\n}\n";
    let mut f2 = File::create(pipe_dir.join("schema.sysml"))?;
    write!(f2, "{}", sysml2)?;

    let res = check_dual_schema_ssot_parity(&tmp.path);
    assert!(res.is_err());
    Ok(())
}

#[test]
fn test_dual_schema_parity_multi_file_passes() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("multi_file")?;
    let sub1 = tmp.path.join("schema").join("sub1");
    let sub2 = tmp.path.join("schema").join("sub2");
    let pipe_dir = tmp.path.join(".pipeline");
    fs::create_dir_all(&sub1)?;
    fs::create_dir_all(&sub2)?;
    fs::create_dir_all(&pipe_dir)?;

    let sysml_a =
        "package Sub1 {\n  part def SensorA;\n  port def PortA;\n  action def ActionA;\n}\n";
    let sysml_b =
        "package Sub2 {\n  part def SensorB;\n  port def PortB;\n  action def ActionB;\n}\n";
    let combined = format!("{}\n{}", sysml_a, sysml_b);

    let mut fa = File::create(sub1.join("a.sysml"))?;
    write!(fa, "{}", sysml_a)?;

    let mut fb = File::create(sub2.join("b.sysml"))?;
    write!(fb, "{}", sysml_b)?;

    let discovered_before_pipeline = discover_sysml_model_text(&tmp.path);
    assert!(
        discovered_before_pipeline.is_some(),
        "discover_sysml_model_text must discover multi-file schema across nested directories"
    );
    let disc_text = discovered_before_pipeline.ok_or("expected discovered text")?;
    assert!(disc_text.contains("SensorA"));
    assert!(disc_text.contains("SensorB"));

    let mut fp = File::create(pipe_dir.join("schema.sysml"))?;
    write!(fp, "{}", combined)?;

    assert_eq!(
        check_dual_schema_ssot_parity(&tmp.path),
        Ok(ParityOutcome::DualSchemaIdentical)
    );
    Ok(())
}

#[test]
fn test_issue_256_check28_rejects_missing_conops_schema_architecture_contract(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("check28_missing_arch")?;
    let schemas_dir = tmp.path.join(".pipeline").join("schemas");
    fs::create_dir_all(&schemas_dir)?;

    let invalid_schema = r#"{
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "required": ["metadata", "operational_context"],
        "properties": {
            "metadata": { "type": "object" },
            "operational_context": { "type": "object" }
        }
    }"#;
    fs::write(
        schemas_dir.join("conops_specification_schema.json"),
        invalid_schema,
    )?;

    let res = check_conops_and_mission_intent_completeness(&tmp.path, false, true);
    assert!(
        res.is_err(),
        "Check 28 must fail when conops schema omits architecture properties"
    );
    let errs = match res {
        Err(e) => e,
        Ok(v) => return Err(format!("expected Err, got {:?}", v).into()),
    };
    assert!(errs.iter().any(|e| e.contains("super_system_architecture")));
    assert!(errs.iter().any(|e| e.contains("subsystem_architecture")));
    Ok(())
}

#[test]
fn test_issue_256_check28_passes_valid_conops_schema_architecture_contract(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("check28_valid_arch")?;
    let schemas_dir = tmp.path.join(".pipeline").join("schemas");
    fs::create_dir_all(&schemas_dir)?;

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or("missing root")?;
    let live_schema = root
        .join(".pipeline")
        .join("schemas")
        .join("conops_specification_schema.json");
    if live_schema.is_file() {
        let content = fs::read_to_string(&live_schema)?;
        fs::write(
            schemas_dir.join("conops_specification_schema.json"),
            content,
        )?;

        let res = check_conops_and_mission_intent_completeness(&tmp.path, false, true);
        assert!(
            res.is_ok(),
            "Check 28 must pass when conops schema contract is complete: {:?}",
            res
        );
    }
    Ok(())
}

#[test]
fn test_issue_271_conops_sv1_icd_parity_mismatch_fails() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("check27_conops_icd_mismatch")?;
    let conops_dir = tmp.path.join("docs").join("conops");
    let interfaces_dir = tmp.path.join("docs").join("interfaces");
    fs::create_dir_all(&conops_dir)?;
    fs::create_dir_all(&interfaces_dir)?;

    let conops_content = r#"## 4. System Operational Architecture
```mermaid
flowchart TD
    Operator -->|"CONN-01: Supervisory Command"| SupervisoryStation
    SupervisoryStation -->|"CONN-02: System Datalink"| Controller
```
"#;
    fs::write(conops_dir.join("04_SYSTEM_ARCHITECTURE.md"), conops_content)?;

    let icd_content = r#"# Level 1C: System Interface Matrix
## 5. Connection Binding Roster Table
| Connection ID | Source Port | Dest Port |
| :--- | :--- | :--- |
| `CONN-01` | PORT-SUP-CMD | PORT-CTRL-CMD |
| `CONN-99` | PORT-ALT-CMD | PORT-CTRL-ALT |
"#;
    fs::write(
        interfaces_dir.join("ICD_01_SYSTEM_INTERFACE_MATRIX.md"),
        icd_content,
    )?;

    let res = check_cross_document_diagram_parity(&tmp.path);
    assert!(
        res.is_err(),
        "Check 27 must fail when ConOps SV-1 and Level 1C ICD connections mismatch"
    );
    Ok(())
}

#[test]
fn test_issue_271_conops_sv1_icd_parity_passes() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("check27_conops_icd_matching")?;
    let conops_dir = tmp.path.join("docs").join("conops");
    let interfaces_dir = tmp.path.join("docs").join("interfaces");
    fs::create_dir_all(&conops_dir)?;
    fs::create_dir_all(&interfaces_dir)?;

    let conops_content = r#"## 4. System Operational Architecture
```mermaid
flowchart TD
    Operator -->|"CONN-01: Supervisory Command"| SupervisoryStation
    SupervisoryStation -->|"CONN-02: System Datalink"| Controller
```
"#;
    fs::write(conops_dir.join("04_SYSTEM_ARCHITECTURE.md"), conops_content)?;

    let icd_content = r#"# Level 1C: System Interface Matrix
## 5. Connection Binding Roster Table
| Connection ID | Source Port | Dest Port |
| :--- | :--- | :--- |
| `CONN-01` | PORT-SUP-CMD | PORT-CTRL-CMD |
| `CONN-02` | PORT-SUP-TLM | PORT-CTRL-TLM |
"#;
    fs::write(
        interfaces_dir.join("ICD_01_SYSTEM_INTERFACE_MATRIX.md"),
        icd_content,
    )?;

    let res = check_cross_document_diagram_parity(&tmp.path);
    assert_eq!(res, Ok(ParityOutcome::Verified));
    Ok(())
}

#[test]
fn test_defect_276_divergent_reports_diagram_without_ssot_views_library_fails(
) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("check27_reports_divergence")?;
    let conops_dir = tmp.path.join("docs").join("conops");
    let reports_dir = tmp.path.join("docs").join("reports");
    let interfaces_dir = tmp.path.join("docs").join("interfaces");
    fs::create_dir_all(&conops_dir)?;
    fs::create_dir_all(&reports_dir)?;
    fs::create_dir_all(&interfaces_dir)?;

    let conops_content = r#"## 4. System Operational Architecture
```mermaid
flowchart TD
    Operator -->|"CONN-01: Supervisory Command"| SupervisoryStation
    SupervisoryStation -->|"CONN-02: System Datalink"| Controller
```
"#;
    fs::write(conops_dir.join("04_SYSTEM_ARCHITECTURE.md"), conops_content)?;

    let icd_content = r#"# Level 1C: System Interface Matrix
| Connection ID | Source Port | Dest Port |
| `CONN-01` | PORT-SUP-CMD | PORT-CTRL-CMD |
| `CONN-02` | PORT-SUP-TLM | PORT-CTRL-TLM |
"#;
    fs::write(
        interfaces_dir.join("ICD_01_SYSTEM_INTERFACE_MATRIX.md"),
        icd_content,
    )?;

    let report_content = r#"# Executive Engineering Deliverables
```mermaid
flowchart TD
    SubA -->|"CONN-99: Divergent Unaligned Flow"| SubB
```
"#;
    fs::write(
        reports_dir.join("PHASE_1_EXECUTIVE_ENGINEERING_DELIVERABLES.md"),
        report_content,
    )?;

    let res = check_cross_document_diagram_parity(&tmp.path);
    assert!(
        res.is_err(),
        "Check 27 must fail when executive deliverables report contains divergent diagram connections"
    );
    Ok(())
}

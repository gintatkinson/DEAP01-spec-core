//! Unit tests for `checks::safety_stpa` (AST closure, STPA_MATRIX, and fail-closed traversal).
//! Realises: [REQ-CHECKS-SAFETY-STPA]

use std::fs;

use crate::checks::safety_stpa::{check_safety_integrity, safety_tests_a::TempDir};

#[test]
fn test_issue_249_check17_rejects_fmeca_with_undeclared_or_negated_component(
) -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new("check17_issue249")?;
    let safety_dir = temp.path.join("docs").join("safety");
    let schema_dir = temp.path.join("schema");
    fs::create_dir_all(&safety_dir)?;
    fs::create_dir_all(&schema_dir)?;

    let model_text = r#"package SystemSSOT {
    part def PrimaryProcessor;
    part def SensorUnit;
    attribute backupEnabled = 0;
    attribute backupSystem = "No";
}
"#;
    fs::write(schema_dir.join("model.sysml"), model_text)?;

    let mut content = String::from("# System Safety & STPA Matrix\n\n");
    content.push_str("## 1. System Losses ($L-1..N$)\n- Loss L-1: Loss of System Function\n\n");
    content.push_str("## 2. System Hazards ($H-1..N$)\n- Hazard H-1: Loss of Control\n\n");
    content.push_str("## 3. Control Structure Topology\n```mermaid\ngraph TD\n  A --> B\n```\n\n");
    content.push_str("## 4. Unsafe Control Actions ($UCA-1..N$)\n- UCA-1: Providing Halt command too late\n- UCA-2: Not providing Halt command\n- UCA-3: Stopped too soon\n- UCA-4: Applied too long\n\n");
    content.push_str("## 5. Loss Scenarios ($LS-1..N$)\n- Scenario LS-1: Sensor failure causes loss\n\n");
    content.push_str("## 6. Formal Safety Constraints ($SC-1..N$)\n- SC-01: System shall remain within safe operating envelope\n\n");
    content.push_str("## 7. FMECA Criticality Matrix\n\n");
    content.push_str("| FMECA ID | Component | Failure Mode | Potential Effect | Severity (S) | Occurrence (O) | Detection (D) | RPN (S x O x D) | Mitigation | Derivation Basis |\n");
    content.push_str("| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n");
    for i in 1..=8 {
        content.push_str(&format!("| FMECA-{:03} | PrimaryProcessor | Mode {} (Interface Γ, State Φ) | Effect | 5 | 2 | 2 | 20 | Mit | SSOT |\n", i, i));
    }
    for i in 9..=14 {
        content.push_str(&format!("| FMECA-{:03} | SensorUnit | Mode {} (Action Ω, Resource Ψ) | Effect | 5 | 2 | 2 | 20 | Mit | SSOT |\n", i, i));
    }
    content.push_str("| FMECA-015 | AuxiliaryBackupUnit | Activation Failure (Action Ω) | Loss of redundancy | 10 | 2 | 2 | 40 | Watchdog | Derived |\n");
    content.push_str("| FMECA-016 | AuxiliaryBackupUnit | Supply Depletion (Resource Ψ) | Loss of redundancy | 10 | 2 | 2 | 40 | Dual Supply | Derived |\n\n");
    content.push_str("## 8. Domain Regulatory Objectives & Integrity Mapping\n- Pillar 8 SafetyIntegrityLevel_2 objectives\n\n");
    content.push_str("## Commercial Toolchain Hooks\n- MATLAB / Simulink / Stateflow hooks\n");

    fs::write(safety_dir.join("STPA_MATRIX.md"), content)?;

    let result = check_safety_integrity(&temp.path, false, true);
    assert!(result.is_err());
    let errs = result.err().ok_or("expected Err")?;
    assert!(errs.iter().any(|e| e.contains("Pillar 7 violation")
        && e.contains("'AuxiliaryBackupUnit' contradicted by negative AST model constraint")));
    assert!(!errs.iter().any(|e| e.contains("contradicted by negative AST model constraint")
        && (e.contains("'PrimaryProcessor'") || e.contains("'SensorUnit'"))));
    Ok(())
}

#[test]
fn test_issue_249_check17_rejects_missing_declared_part() -> Result<(), Box<dyn std::error::Error>>
{
    let temp = TempDir::new("check17_missing_part")?;
    let safety_dir = temp.path.join("docs").join("safety");
    let schema_dir = temp.path.join("schema");
    fs::create_dir_all(&safety_dir)?;
    fs::create_dir_all(&schema_dir)?;

    let model_text = r#"package SystemSSOT {
    part def FlightComputer;
    part def ActuatorUnit;
}
"#;
    fs::write(schema_dir.join("model.sysml"), model_text)?;

    let mut content = String::from("# System Safety & STPA Matrix\n\n");
    content.push_str("## 1. System Losses ($L-1..N$)\n- Loss L-1: Loss of Vehicle\n\n");
    content.push_str("## 2. System Hazards ($H-1..N$)\n- Hazard H-1: Loss of Control\n\n");
    content.push_str("## 3. Control Structure Topology\n```mermaid\ngraph TD\n  A --> B\n```\n\n");
    content.push_str("## 4. Unsafe Control Actions ($UCA-1..N$)\n- UCA-1: Providing Land command too late\n- UCA-2: Not providing Land command\n- UCA-3: Stopped too soon\n- UCA-4: Applied too long\n\n");
    content.push_str("## 5. Loss Scenarios ($LS-1..N$)\n- Scenario LS-1: Sensor failure causes loss\n\n");
    content.push_str("## 6. Formal Safety Constraints ($SC-1..N$)\n- SC-01: Vehicle shall remain in safe corridor\n\n");
    content.push_str("## 7. FMECA Criticality Matrix\n\n");
    content.push_str("| FMECA ID | Component | Failure Mode | Potential Effect | Severity (S) | Occurrence (O) | Detection (D) | RPN (S x O x D) | Mitigation | Derivation Basis |\n");
    content.push_str("| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n");
    for i in 1..=15 {
        let dim = match i % 4 {
            0 => "Interface Γ",
            1 => "State Φ",
            2 => "Action Ω",
            _ => "Resource Ψ",
        };
        content.push_str(&format!(
            "| FMECA-{:03} | FlightComputer | Mode {} ({}) | Effect | 5 | 2 | 2 | 20 | Mit | SSOT |\n",
            i, i, dim
        ));
    }
    content.push_str("\n## 8. Domain Regulatory Objectives & Integrity Mapping\n- Pillar 8 SORA SAIL II objectives\n\n");
    content.push_str("## Commercial Toolchain Hooks\n- MATLAB / Simulink / Stateflow hooks\n");

    fs::write(safety_dir.join("STPA_MATRIX.md"), content)?;

    let result = check_safety_integrity(&temp.path, false, true);
    assert!(result.is_err());
    let errs = result.err().ok_or("expected Err")?;
    assert!(errs
        .iter()
        .any(|e| e.contains("missing declared AST part def component(s): ActuatorUnit")));
    Ok(())
}

#[test]
fn test_issue_249_check17_valid_bidirectional_ast_closure_passes(
) -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new("check17_valid_ast")?;
    let safety_dir = temp.path.join("docs").join("safety");
    let schema_dir = temp.path.join("schema");
    fs::create_dir_all(&safety_dir)?;
    fs::create_dir_all(&schema_dir)?;

    let model_text = r#"package SystemSSOT {
    part def FlightComputer;
    part def SensorUnit;
}
"#;
    fs::write(schema_dir.join("model.sysml"), model_text)?;

    let mut content = String::from("# System Safety & STPA Matrix\n\n");
    content.push_str("## 1. System Losses ($L-1..N$)\n- Loss L-1: Loss of Vehicle\n\n");
    content.push_str("## 2. System Hazards ($H-1..N$)\n- Hazard H-1: Loss of Control\n\n");
    content.push_str("## 3. Control Structure Topology\n```mermaid\ngraph TD\n  A --> B\n```\n\n");
    content.push_str("## 4. Unsafe Control Actions ($UCA-1..N$)\n- UCA-1: Providing Land command too late\n- UCA-2: Not providing Land command\n- UCA-3: Stopped too soon\n- UCA-4: Applied too long\n\n");
    content.push_str("## 5. Loss Scenarios ($LS-1..N$)\n- Scenario LS-1: Sensor failure causes loss\n\n");
    content.push_str("## 6. Formal Safety Constraints ($SC-1..N$)\n- SC-01: Vehicle shall remain in safe corridor\n\n");
    content.push_str("## 7. FMECA Criticality Matrix\n\n");
    content.push_str("| FMECA ID | Component | Failure Mode | Potential Effect | Severity (S) | Occurrence (O) | Detection (D) | RPN (S x O x D) | Mitigation | Derivation Basis |\n");
    content.push_str("| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n");
    for i in 1..=8 {
        let dim = match i % 4 {
            0 => "Interface Γ",
            1 => "State Φ",
            2 => "Action Ω",
            _ => "Resource Ψ",
        };
        content.push_str(&format!(
            "| FMECA-{:03} | FlightComputer | Mode {} ({}) | Effect | 5 | 2 | 2 | 20 | Mit | SSOT |\n",
            i, i, dim
        ));
    }
    for i in 9..=16 {
        let dim = match i % 4 {
            0 => "Interface Γ",
            1 => "State Φ",
            2 => "Action Ω",
            _ => "Resource Ψ",
        };
        content.push_str(&format!(
            "| FMECA-{:03} | SensorUnit | Mode {} ({}) | Effect | 5 | 2 | 2 | 20 | Mit | Derived |\n",
            i, i, dim
        ));
    }
    content.push_str("\n## 8. Domain Regulatory Objectives & Integrity Mapping\n- Pillar 8 SORA SAIL II objectives\n\n");
    content.push_str("## Commercial Toolchain Hooks\n- MATLAB / Simulink / Stateflow hooks\n");

    fs::write(safety_dir.join("STPA_MATRIX.md"), content)?;

    let result = check_safety_integrity(&temp.path, false, true);
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_defect_333_missing_stpa_matrix_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new("check17_missing_stpa")?;
    let safety_dir = temp.path.join("docs").join("safety");
    fs::create_dir_all(&safety_dir)?;

    fs::write(safety_dir.join("01_LOSSES.md"), "# Losses\nL-1: Loss\n")?;
    fs::write(safety_dir.join("05_FMECA.md"), "# FMECA\n| FM | Comp | Mode |\n")?;

    let result = check_safety_integrity(&temp.path, false, true);
    assert!(result.is_err());
    let errs = result.err().ok_or("expected Err")?;
    assert!(errs
        .iter()
        .any(|e| e.contains("Unified STPA matrix 'docs/safety/STPA_MATRIX.md' is missing")));
    Ok(())
}

#[test]
fn test_defect_333_hollow_stpa_matrix_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new("check17_hollow_stpa")?;
    let safety_dir = temp.path.join("docs").join("safety");
    fs::create_dir_all(&safety_dir)?;

    let hollow_content = "# STPA Cross-Traceability Matrix\n\n\
| UCA ID | Controller | Control Action | Guide Word |\n\
|---|---|---|---|\n\
| UCA-001 | Sys | Action | Not providing |\n";
    fs::write(safety_dir.join("STPA_MATRIX.md"), hollow_content)?;

    let result = check_safety_integrity(&temp.path, false, true);
    assert!(result.is_err());
    let errs = result.err().ok_or("expected Err")?;
    assert!(errs.iter().any(|e| e.contains("Pillar 1 violation")));
    Ok(())
}

#[cfg(unix)]
#[test]
fn test_check17_unreadable_safety_subdir_fails_closed() -> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::PermissionsExt;

    let temp = TempDir::new("check17_unreadable_sub")?;
    fs::create_dir_all(temp.path.join(".pipeline/upstream"))?;
    let locked = temp.path.join("docs/safety/locked_subdir");
    fs::create_dir_all(&locked)?;
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o000))?;

    let upstream_res = check_safety_integrity(&temp.path, true, false);
    let _ = fs::set_permissions(&locked, fs::Permissions::from_mode(0o755));

    assert!(upstream_res.is_err());
    Ok(())
}

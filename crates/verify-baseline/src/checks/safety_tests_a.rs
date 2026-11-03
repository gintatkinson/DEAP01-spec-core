//! Unit tests for `checks::safety_stpa` (FMECA multi-mode and basis checks).
//! Realises: [REQ-CHECKS-SAFETY-STPA]

use std::fs;

use crate::checks::safety_stpa::{
    check_safety_integrity, check_uca_categories, SafetyCheckOutcome,
};

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
        let path = std::env::temp_dir().join(format!("verify_safety_{}_{}_{}", name, nanos, count));
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
fn test_missing_safety_dir_allow_missing() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("allow_missing")?;
    let res = check_safety_integrity(&tmp.path, true, false);
    assert_eq!(res, Ok(SafetyCheckOutcome::DownstreamPendingOrClean));
    Ok(())
}

#[test]
fn test_missing_safety_dir_strict() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("strict")?;
    let res = check_safety_integrity(&tmp.path, true, true);
    assert!(res.is_err());
    Ok(())
}

#[test]
fn test_uca_categories() {
    let clean = "Not providing command causes hazard. Providing incorrect command. Too early or out of order. Stopped too soon or applied too long.";
    assert!(check_uca_categories(clean).is_empty());

    let incomplete = "Only omission causes hazard.";
    let missing = check_uca_categories(incomplete);
    assert_eq!(missing.len(), 3);
}

#[test]
fn test_fmeca_single_failure_mode_per_component_rejected() -> Result<(), Box<dyn std::error::Error>>
{
    let tmp = TempDir::new("single_mode")?;
    let safety_dir = tmp.path.join("docs").join("safety");
    fs::create_dir_all(&safety_dir)?;

    let mut fmeca_content = String::from(
        "# Safety Analysis\n\n\
         ## Losses\nL-1: Loss of vehicle\n\n\
         ## Hazards\nH-1: Hazard 1\n\n\
         ## Control Structure\nController and actuator interface.\n\n\
         ## Unsafe Control Actions\n\
         UCA-1: Not providing command causes hazard. Providing causes hazard. Too early or out of order. Stopped too soon or applied too long.\n\n\
         ## Loss Scenarios\nLS-1: Scenario\n\n\
         ## Safety Constraints\nSC-1: Constraint\n\n\
         ## Pillar 7: FMECA Criticality Matrix\n\n\
         | FMECA ID | Component | Failure Mode | Severity (S) | Occurrence (O) | Detection (D) | RPN | Derivation Basis |\n\
         |---|---|---|---|---|---|---|---|\n",
    );
    let components = [
        ("Comp1", "Interface bus loss"),
        ("Comp2", "State transition deadlock"),
        ("Comp3", "Action timing latency"),
        ("Comp4", "Resource power fault"),
        ("Comp5", "Interface frame dropout"),
        ("Comp6", "State desync lockup"),
        ("Comp7", "Action command overshoot"),
        ("Comp8", "Resource thermal overheat"),
        ("Comp9", "Interface signal noise"),
        ("Comp10", "State corruption error"),
        ("Comp11", "Action computation delay"),
        ("Comp12", "Resource memory overflow"),
        ("Comp13", "Interface packet timeout"),
        ("Comp14", "State abort trip"),
        ("Comp15", "Action rate limiting"),
        ("Comp16", "Resource battery sag"),
    ];
    for (i, (comp, mode)) in components.iter().enumerate() {
        fmeca_content.push_str(&format!(
            "| FM-{:03} | {} | {} | 2 | 2 | 2 | 8 | Derived |\n",
            i + 1,
            comp,
            mode
        ));
    }
    fmeca_content.push_str(
        "\n## Pillar 8: Regulatory Objectives\n| Objective ID | Requirement | Allocation |\n|---|---|---|\n| OBJ-1 | Req 1 | Comp1 |\n\nMATLAB Simulink Stateflow Embedded Coder hooks.\n",
    );
    fs::write(safety_dir.join("STPA_MATRIX.md"), fmeca_content)?;

    let res = check_safety_integrity(&tmp.path, false, true);
    assert!(res.is_err());
    Ok(())
}

#[test]
fn test_fmeca_multi_failure_mode_per_component_passes() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("multi_mode")?;
    let safety_dir = tmp.path.join("docs").join("safety");
    fs::create_dir_all(&safety_dir)?;

    let mut fmeca_content = String::from(
        "# Safety Analysis\n\n\
         ## Losses\nL-1: Loss of vehicle\n\n\
         ## Hazards\nH-1: Hazard 1\n\n\
         ## Control Structure\nController and actuator interface.\n\n\
         ## Unsafe Control Actions\n\
         UCA-1: Not providing command causes hazard. Providing causes hazard. Too early or out of order. Stopped too soon or applied too long.\n\n\
         ## Loss Scenarios\nLS-1: Scenario\n\n\
         ## Safety Constraints\nSC-1: Constraint\n\n\
         ## Pillar 7: FMECA Criticality Matrix\n\n\
         | FMECA ID | Component | Failure Mode | Severity (S) | Occurrence (O) | Detection (D) | RPN | Derivation Basis |\n\
         |---|---|---|---|---|---|---|---|\n",
    );
    let component_pairs = [
        ("Comp1", "Interface bus dropout", "State desync lockup"),
        ("Comp2", "Action timing latency", "Resource power fault"),
        ("Comp3", "Interface frame corruption", "State transition freeze"),
        ("Comp4", "Action command overshoot", "Resource thermal overheat"),
        ("Comp5", "Interface packet loss", "State corruption error"),
        ("Comp6", "Action rate limiting", "Resource battery sag"),
        ("Comp7", "Interface bus jitter", "State abort trip"),
        ("Comp8", "Action torque clamping", "Resource memory exhaustion"),
    ];
    let mut row_idx = 1;
    for (comp, mode1, mode2) in &component_pairs {
        fmeca_content.push_str(&format!(
            "| FM-{:03} | {} | {} | 2 | 2 | 2 | 8 | Derived |\n",
            row_idx, comp, mode1
        ));
        row_idx += 1;
        fmeca_content.push_str(&format!(
            "| FM-{:03} | {} | {} | 2 | 2 | 2 | 8 | Derived |\n",
            row_idx, comp, mode2
        ));
        row_idx += 1;
    }
    fmeca_content.push_str(
        "\n## Pillar 8: Regulatory Objectives\n| Objective ID | Requirement | Allocation |\n|---|---|---|\n| OBJ-1 | Req 1 | Comp1 |\n\nMATLAB Simulink Stateflow Embedded Coder hooks.\n",
    );
    fs::write(safety_dir.join("STPA_MATRIX.md"), fmeca_content)?;

    let res = check_safety_integrity(&tmp.path, false, true);
    assert_eq!(res, Ok(SafetyCheckOutcome::DownstreamVerified));
    Ok(())
}

#[test]
fn test_fmeca_partial_single_mode_component_fails() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("partial_single")?;
    let safety_dir = tmp.path.join("docs").join("safety");
    fs::create_dir_all(&safety_dir)?;

    let mut fmeca_content = String::from(
        "# Safety Analysis\n\n\
         ## Losses\nL-1: Loss of vehicle\n\n\
         ## Hazards\nH-1: Hazard 1\n\n\
         ## Control Structure\nController and actuator interface.\n\n\
         ## Unsafe Control Actions\n\
         UCA-1: Not providing command causes hazard. Providing causes hazard. Too early or out of order. Stopped too soon or applied too long.\n\n\
         ## Loss Scenarios\nLS-1: Scenario\n\n\
         ## Safety Constraints\nSC-1: Constraint\n\n\
         ## Pillar 7: FMECA Criticality Matrix\n\n\
         | FMECA ID | Component | Failure Mode | Severity (S) | Occurrence (O) | Detection (D) | RPN | Derivation Basis |\n\
         |---|---|---|---|---|---|---|---|\n",
    );
    for i in 1..=7 {
        fmeca_content.push_str(&format!(
            "| FM-{:03} | Comp{} | Interface bus loss {} | 2 | 2 | 2 | 8 | Derived |\n\
             | FM-{:03} | Comp{} | State desync lockup {} | 2 | 2 | 2 | 8 | Derived |\n",
            (i - 1) * 2 + 1,
            i,
            i,
            (i - 1) * 2 + 2,
            i,
            i
        ));
    }
    fmeca_content.push_str(
        "| FM-015 | Comp8 | Action command latency and resource power fault | 2 | 2 | 2 | 8 | Derived |\n\n\
         ## Pillar 8: Regulatory Objectives\n| Objective ID | Requirement | Allocation |\n|---|---|---|\n| OBJ-1 | Req 1 | Comp1 |\n\nMATLAB Simulink Stateflow Embedded Coder hooks.\n",
    );
    fs::write(safety_dir.join("STPA_MATRIX.md"), fmeca_content)?;

    let res = check_safety_integrity(&tmp.path, false, true);
    assert!(res.is_err());
    let errs = res.err().ok_or("expected Err")?;
    assert!(errs
        .iter()
        .any(|e| e.contains("Comp8") && e.contains("degenerate single-failure-mode coverage")));
    Ok(())
}

#[test]
fn test_fmeca_duplicate_mode_does_not_satisfy_multi_mode() -> Result<(), Box<dyn std::error::Error>>
{
    let tmp = TempDir::new("duplicate_mode")?;
    let safety_dir = tmp.path.join("docs").join("safety");
    fs::create_dir_all(&safety_dir)?;

    let mut fmeca_content = String::from(
        "# Safety Analysis\n\n\
         ## Losses\nL-1: Loss of vehicle\n\n\
         ## Hazards\nH-1: Hazard 1\n\n\
         ## Control Structure\nController and actuator interface.\n\n\
         ## Unsafe Control Actions\n\
         UCA-1: Not providing command causes hazard. Providing causes hazard. Too early or out of order. Stopped too soon or applied too long.\n\n\
         ## Loss Scenarios\nLS-1: Scenario\n\n\
         ## Safety Constraints\nSC-1: Constraint\n\n\
         ## Pillar 7: FMECA Criticality Matrix\n\n\
         | FMECA ID | Component | Failure Mode | Severity (S) | Occurrence (O) | Detection (D) | RPN | Derivation Basis |\n\
         |---|---|---|---|---|---|---|---|\n\
         | FM-001 | Comp1 | Interface bus dropout | 2 | 2 | 2 | 8 | Derived |\n\
         | FM-002 | Comp1 | Interface bus dropout | 2 | 2 | 2 | 8 | Derived |\n",
    );
    let pairs = [
        ("Comp2", "Action timing latency", "Resource power fault"),
        ("Comp3", "Interface frame corruption", "State transition freeze"),
        ("Comp4", "Action command overshoot", "Resource thermal overheat"),
        ("Comp5", "Interface packet loss", "State corruption error"),
        ("Comp6", "Action rate limiting", "Resource battery sag"),
        ("Comp7", "Interface bus jitter", "State abort trip"),
        ("Comp8", "Action torque clamping", "Resource memory exhaustion"),
    ];
    let mut row_idx = 3;
    for (comp, mode1, mode2) in &pairs {
        fmeca_content.push_str(&format!(
            "| FM-{:03} | {} | {} | 2 | 2 | 2 | 8 | Derived |\n",
            row_idx, comp, mode1
        ));
        row_idx += 1;
        fmeca_content.push_str(&format!(
            "| FM-{:03} | {} | {} | 2 | 2 | 2 | 8 | Derived |\n",
            row_idx, comp, mode2
        ));
        row_idx += 1;
    }
    fmeca_content.push_str(
        "\n## Pillar 8: Regulatory Objectives\n| Objective ID | Requirement | Allocation |\n|---|---|---|\n| OBJ-1 | Req 1 | Comp1 |\n\nMATLAB Simulink Stateflow Embedded Coder hooks.\n",
    );
    fs::write(safety_dir.join("STPA_MATRIX.md"), fmeca_content)?;

    let res = check_safety_integrity(&tmp.path, false, true);
    assert!(res.is_err());
    let errs = res.err().ok_or("expected Err")?;
    assert!(errs.iter().any(|e| e.contains("Comp1")));
    Ok(())
}

#[test]
fn test_fmeca_missing_derivation_basis_fails() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new("missing_basis")?;
    let safety_dir = tmp.path.join("docs").join("safety");
    fs::create_dir_all(&safety_dir)?;

    let mut fmeca_content = String::from(
        "# Safety Analysis\n\n\
         ## Losses\nL-1: Loss of vehicle\n\n\
         ## Hazards\nH-1: Hazard 1\n\n\
         ## Control Structure\nController and actuator interface.\n\n\
         ## Unsafe Control Actions\n\
         UCA-1: Not providing command causes hazard. Providing causes hazard. Too early or out of order. Stopped too soon or applied too long.\n\n\
         ## Loss Scenarios\nLS-1: Scenario\n\n\
         ## Safety Constraints\nSC-1: Constraint\n\n\
         ## Pillar 7: FMECA Criticality Matrix\n\n\
         | FMECA ID | Component | Failure Mode | Severity (S) | Occurrence (O) | Detection (D) | RPN | Derivation Basis |\n\
         |---|---|---|---|---|---|---|---|\n",
    );
    let pairs = [
        ("Comp1", "Interface bus dropout", "State desync lockup"),
        ("Comp2", "Action timing latency", "Resource power fault"),
        ("Comp3", "Interface frame corruption", "State transition freeze"),
        ("Comp4", "Action command overshoot", "Resource thermal overheat"),
        ("Comp5", "Interface packet loss", "State corruption error"),
        ("Comp6", "Action rate limiting", "Resource battery sag"),
        ("Comp7", "Interface bus jitter", "State abort trip"),
        ("Comp8", "Action torque clamping", "Resource memory exhaustion"),
    ];
    let mut row_idx = 1;
    for (comp, mode1, mode2) in &pairs {
        let basis1 = if row_idx == 1 { "" } else { "Derived" };
        fmeca_content.push_str(&format!(
            "| FM-{:03} | {} | {} | 2 | 2 | 2 | 8 | {} |\n",
            row_idx, comp, mode1, basis1
        ));
        row_idx += 1;
        fmeca_content.push_str(&format!(
            "| FM-{:03} | {} | {} | 2 | 2 | 2 | 8 | Derived |\n",
            row_idx, comp, mode2
        ));
        row_idx += 1;
    }
    fmeca_content.push_str(
        "\n## Pillar 8: Regulatory Objectives\n| Objective ID | Requirement | Allocation |\n|---|---|---|\n| OBJ-1 | Req 1 | Comp1 |\n\nMATLAB Simulink Stateflow Embedded Coder hooks.\n",
    );
    fs::write(safety_dir.join("STPA_MATRIX.md"), fmeca_content)?;

    let res = check_safety_integrity(&tmp.path, false, true);
    assert!(res.is_err());
    let errs = res.err().ok_or("expected Err")?;
    assert!(errs
        .iter()
        .any(|e| e.contains("missing explicit Derivation Basis classification")));
    Ok(())
}

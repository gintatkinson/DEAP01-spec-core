"""
Safety Integrity Quality Gate & SORA OSO-01..24 Completeness Verification Suite.
/// Realises: [SafetyIntegrityQualityGate, SORACompleteness, ASTM_F3269_RTA]
"""
import os
import sys
import tempfile
import pytest

# Ensure scripts directory is in sys.path
repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if repo_root not in sys.path:
    sys.path.insert(0, repo_root)

from scripts.verify_downstream_baseline import (
    count_fmeca_rows,
    check_uca_categories,
    check_sora_osos,
    validate_safety_matrix_content,
    check_safety_integrity_and_sora_completeness,
)


def generate_valid_stpa_matrix_content(fmeca_row_count=16, include_all_osos=True):
    """Generate a fully conforming 8-pillar STPA_MATRIX.md string."""
    fmeca_rows = []
    for i in range(1, fmeca_row_count + 1):
        fmeca_rows.append(
            f"| FM-{i:02d} | Subsystem-{i} | Failure Mode {i} | Local Effect {i} | System Loss L-1 | 4 | 2 | 2 | 16 | Redundant Channel {i} |"
        )
    fmeca_table_str = "\n".join(fmeca_rows)

    osos_list = [f"- **OSO-{i:02d}**: Robustness Level High / Satisfied via Architecture" for i in range(1, 25)]
    if not include_all_osos:
        osos_list = osos_list[:-2]  # Remove OSO-23 and OSO-24
    osos_str = "\n".join(osos_list)

    header_suffix = "(OSO-01 through OSO-24)" if include_all_osos else "(Partial OSO Set)"
    return rf"""# STPA Safety Analysis, FMECA Matrix & SORA SAIL Assessment

> **Primary Commercial Toolchain Integration Context:** MATLAB / Simulink / Stateflow / Embedded Coder  
> **Safety Standards:** JARUS SORA v2.5 | ASTM F3269-17 RTA | RTCA DO-365B  

---

## 1. System Losses (**L-1..N**)

- **L-1**: Loss of human life or severe ground fatal injury.
- **L-2**: Mid-air collision with crewed aircraft.
- **L-3**: Total loss of UAS airframe and critical infrastructure payload.

---

## 2. System Hazards (**H-1..N**)

- **H-1**: Aircraft breaches 3D operational containment geofence boundary.
- **H-2**: Aircraft violates RTCA DO-365B DAA well-clear safety separation.
- **H-3**: Uncontrolled flight termination due to propulsion/actuator loss.

---

## 3. Hierarchical Control Structure Topology

The control structure consists of the Remote Pilot in Command (RPIC), Autopilot Flight Controller, ASTM F3269-17 Run-Time Assurance (RTA) Safety Net Monitor, Actuator Servos, and Telemetry Sensor Suite.

```mermaid
flowchart TD
    RPIC["Remote Pilot in Command"] --> Autopilot["Autopilot Flight Controller"]
    Autopilot --> RTA["ASTM F3269-17 RTA Monitor"]
    RTA --> Actuator["Actuator Servos / Flight Surfaces"]
    Sensors["IMU / GPS / DAA Sensors"] --> RTA
    Sensors --> Autopilot
```

---

## 4. Unsafe Control Actions (**UCA-1..N**)

Systematic identification across 4 STPA guide words / failure mode categories:

1. **Not providing causes hazard**:
   - `UCA-01`: Not providing emergency parachute deployment command when uncontrolled descent detected.
2. **Providing causes hazard**:
   - `UCA-02`: Providing motor cutoff command during active low-altitude hover over populated area.
3. **Providing too early, too late, or out of order**:
   - `UCA-03`: Providing collision avoidance maneuver too late after DAA boundary violation.
4. **Stopped too soon or applied too long**:
   - `UCA-04`: Stopped too soon contingency Return-to-Launch climb before reaching minimum safe altitude.

---

## 5. Loss Scenarios (**LS-1..N**) & Causal Factors

- **LS-1**: Primary GNSS spoofing causes false position estimation, leading to geofence boundary breach (**H-1**, **L-1**).
- **LS-2**: Actuator telemetry packet loss stalls flight control surface transition.

---

## 6. Formal Safety Constraints (**SC-1..N**)

- **SC-1**: The flight control system shall enforce pitch limits between $-15^\circ$ and $+25^\circ$ under all operating conditions.
- **SC-2**: The ASTM F3269-17 RTA Safety Net shall switch to certified safe-state recovery within 50ms of barrier violation.

---

## 7. FMECA Criticality Matrix

| Failure ID | Component / Subsystem | Failure Mode | Local Effect | System Effect | S | O | D | RPN | Mitigating Design Control |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
{fmeca_table_str}

---

## 8. SORA SAIL Risk Mitigations & OSO Traceability Table

- **Ground Risk Class (GRC):** Final GRC = 4 (Initial GRC = 5, M1/M2 mitigations applied).
- **Air Risk Class (ARC):** Final ARC-c.
- **Specific Assurance and Integrity Level (SAIL):** SAIL III.

### Operational Safety Objectives {header_suffix}

{osos_str}

---

## 9. ASTM F3269-17 Run-Time Assurance (RTA) & Commercial Toolchain Architecture

The safety net monitor architecture complies with **ASTM F3269-17** Run-Time Assurance (RTA) for Aircraft Systems. Formal invariant proofs and Stateflow recovery supervisors are synthesized directly into **MATLAB / Simulink / Stateflow / Embedded Coder** and verified with Simulink Design Verifier (SLDV).
"""


def test_upstream_safety_landing_zone_clean():
    """Verify that upstream distribution templates enforce clean docs/safety/ landing zone."""
    if os.path.isdir(os.path.join(repo_root, ".pipeline", "upstream")):
        safety_dir = os.path.join(repo_root, "docs", "safety")
        if os.path.isdir(safety_dir):
            allowed = {".gitkeep", "README.md"}
            for f in os.listdir(safety_dir):
                assert f in allowed, f"Upstream template contains non-template file in docs/safety/: {f}"


def test_upstream_safety_landing_zone_dirty_fails():
    """Verify check_safety_integrity_and_sora_completeness rejects dirty upstream safety landing zones."""
    with tempfile.TemporaryDirectory() as tmpdir:
        os.makedirs(os.path.join(tmpdir, ".pipeline", "upstream"), exist_ok=True)
        os.makedirs(os.path.join(tmpdir, "docs", "safety"), exist_ok=True)

        # Write allowed README.md
        with open(os.path.join(tmpdir, "docs", "safety", "README.md"), "w") as f:
            f.write("# Safety Directory\n")

        # Write concrete spec file (violation)
        with open(os.path.join(tmpdir, "docs", "safety", "STPA_MATRIX.md"), "w") as f:
            f.write("# Concrete STPA Matrix\n")

        with pytest.raises(SystemExit) as exc_info:
            check_safety_integrity_and_sora_completeness(tmpdir)
        assert exc_info.value.code == 1


def test_downstream_8_pillar_passing():
    """Verify that a complete 8-pillar STPA matrix passes validation with zero errors."""
    valid_content = generate_valid_stpa_matrix_content(fmeca_row_count=16, include_all_osos=True)
    errors = validate_safety_matrix_content(valid_content)
    assert not errors, f"Expected 0 errors for valid 8-pillar STPA matrix, got:\n{errors}"


def test_sora_oso_01_to_24_validation():
    """Verify all 24 SORA OSOs (OSO-01 through OSO-24) are rigorously validated."""
    # Test complete list
    all_osos_text = " ".join([f"OSO-{i:02d}" for i in range(1, 25)])
    assert check_sora_osos(all_osos_text) == []

    # Test missing OSO-07 and OSO-24
    partial_osos_text = " ".join([f"OSO-{i:02d}" for i in range(1, 25) if i not in (7, 24)])
    missing = check_sora_osos(partial_osos_text)
    assert missing == ["OSO-07", "OSO-24"]

    # Test within full document
    incomplete_content = generate_valid_stpa_matrix_content(include_all_osos=False)
    errors = validate_safety_matrix_content(incomplete_content)
    assert any("OSO-23" in err and "OSO-24" in err for err in errors), f"Expected missing OSOs error, got:\n{errors}"


def test_fmeca_row_count_validation():
    """Verify FMECA matrix row count requires at least 15 component rows."""
    valid_content_16 = generate_valid_stpa_matrix_content(fmeca_row_count=16)
    assert count_fmeca_rows(valid_content_16) >= 15
    assert validate_safety_matrix_content(valid_content_16) == []

    # Exactly 15 rows
    valid_content_15 = generate_valid_stpa_matrix_content(fmeca_row_count=15)
    assert count_fmeca_rows(valid_content_15) == 15
    assert validate_safety_matrix_content(valid_content_15) == []

    # Less than 15 rows (e.g. 5 rows)
    invalid_content_5 = generate_valid_stpa_matrix_content(fmeca_row_count=5)
    assert count_fmeca_rows(invalid_content_5) == 5
    errors = validate_safety_matrix_content(invalid_content_5)
    assert any("FMECA Criticality Matrix contains 5 row(s); minimum required is 15 rows" in err for err in errors)


def test_uca_failure_mode_categories():
    """Verify all 4 STPA UCA failure mode categories are required."""
    all_cats_text = (
        "1. Not providing causes hazard\n"
        "2. Providing causes hazard\n"
        "3. Providing too early, too late, or out of order\n"
        "4. Stopped too soon or applied too long"
    )
    assert check_uca_categories(all_cats_text) == []

    # Missing "Not providing"
    no_omission = (
        "2. Providing causes hazard\n"
        "3. Providing too early, too late, or out of order\n"
        "4. Stopped too soon or applied too long"
    )
    missing = check_uca_categories(no_omission)
    assert any("Not providing" in m for m in missing)


def test_astm_f3269_rta_and_commercial_toolchain_hooks():
    """Verify ASTM F3269-17 RTA and MATLAB/Simulink hooks are strictly enforced."""
    base_content = generate_valid_stpa_matrix_content()

    # Strip ASTM F3269-17
    no_rta = base_content.replace("ASTM F3269-17", "").replace("ASTM F3269", "")
    errors = validate_safety_matrix_content(no_rta)
    assert any("ASTM F3269-17" in err for err in errors)

    # Strip MATLAB / Simulink
    no_matlab = base_content.replace("MATLAB", "").replace("Simulink", "").replace("Stateflow", "").replace("Embedded Coder", "").replace("SLDV", "")
    errors = validate_safety_matrix_content(no_matlab)
    assert any("MATLAB / Simulink" in err for err in errors)


def test_end_to_end_check_17_downstream_integration():
    """Verify end-to-end Check 17 execution on downstream project directory."""
    with tempfile.TemporaryDirectory() as tmpdir:
        # Downstream project (no .pipeline/upstream)
        safety_dir = os.path.join(tmpdir, "docs", "safety")
        os.makedirs(safety_dir, exist_ok=True)

        stpa_file = os.path.join(safety_dir, "STPA_MATRIX.md")
        valid_content = generate_valid_stpa_matrix_content(fmeca_row_count=16, include_all_osos=True)

        with open(stpa_file, "w", encoding="utf-8") as f:
            f.write(valid_content)

        # Should pass with no exception
        check_safety_integrity_and_sora_completeness(tmpdir)

        # Corrupt file with violation (drop OSO-24)
        corrupted_content = valid_content.replace("OSO-24", "INVALID-REF")
        with open(stpa_file, "w", encoding="utf-8") as f:
            f.write(corrupted_content)

        with pytest.raises(SystemExit) as exc_info:
            check_safety_integrity_and_sora_completeness(tmpdir)
        assert exc_info.value.code == 1

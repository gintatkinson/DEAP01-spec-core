"""
FMECA AST Coverage Gate, Multiplicity & Integer RPN Verification Suite.
/// Realises: [Feat-070/Check17FMECAASTCoverageGate, FMECA_AST_Coverage, BasisClassification]
"""
import os
import sys
import unittest
import pytest

repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if repo_root not in sys.path:
    sys.path.insert(0, repo_root)

from scripts.verify_downstream_baseline import (
    parse_fmeca_table,
    validate_safety_matrix_content,
    validate_safety_matrix_ast,
    check_safety_integrity_and_sora_completeness,
)


def build_test_sysml_model(part_names, actions_per_part=4, total_scs=100):
    """Build a SysML v2 model declaring specified part defs, action defs, and requirement defs."""
    lines = ["package TestSystem {"]
    action_idx = 1
    for part in part_names:
        lines.append(f"    part def {part} {{")
        for _ in range(actions_per_part):
            lines.append(f"        action def Action{action_idx:02d};")
            action_idx += 1
        lines.append("    }")
    for i in range(1, total_scs + 1):
        lines.append(f"    requirement def SafetyConstraint_SC_{i:02d};")
    lines.append("}")
    return "\n".join(lines) + "\n"


def build_test_safety_document(components_modes_map, total_actions=12):
    """Build a complete 8-pillar safety matrix document with custom FMECA rows."""
    uca_rows = []
    uca_idx = 1
    guide_words = [
        "Not providing causes hazard",
        "Providing causes hazard",
        "Providing too early, too late, or out of order",
        "Stopped too soon or applied too long",
    ]
    for action_num in range(1, total_actions + 1):
        for gw in guide_words:
            uca_rows.append(
                f"| UCA-{uca_idx:02d} | Controller | Action{action_num:02d} | {gw} | H-1 | L-1 | SC-{uca_idx:02d} |"
            )
            uca_idx += 1

    uca_table = (
        "| UCA ID | Controller | Control Action | Guide Word | Hazard Ref | System Loss Ref | Safety Constraint |\n"
        "| :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n"
        + "\n".join(uca_rows)
    )

    fmeca_rows = []
    fm_idx = 1
    for comp, modes in components_modes_map.items():
        for mode_entry in modes:
            if isinstance(mode_entry, tuple):
                mode_name, s, o, d, rpn, basis = mode_entry
            else:
                mode_name = mode_entry
                s, o, d, rpn, basis = 4, 2, 2, 16, "SSOT"
            fmeca_rows.append(
                f"| FM-{fm_idx:02d} | {comp} | {mode_name} | Local Effect {fm_idx:02d} | "
                f"System Loss L-1 | {s} | {o} | {d} | {rpn} | Design Control {fm_idx:02d} | {basis} |"
            )
            fm_idx += 1

    fmeca_table = (
        "| Failure ID | Component / Subsystem | Failure Mode | Local Effect | System Effect | S | O | D | RPN | Mitigating Design Control | Basis |\n"
        "| :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n"
        + "\n".join(fmeca_rows)
    )

    oso_table = (
        "| OSO ID | Robustness Level | Justification | Mitigation Ref |\n"
        "| :--- | :--- | :--- | :--- |\n"
        + "\n".join(
            f"| OSO-{i:02d} | Robust | Justification for OSO-{i:02d} | M-{i:02d} |"
            for i in range(1, 25)
        )
    )

    return rf"""# STPA Safety Analysis, FMECA Matrix & SORA SAIL Assessment

> **Primary Commercial Toolchain Integration Context:** MATLAB / Simulink / Stateflow / Embedded Coder
> **Safety Standards:** JARUS SORA v2.5 | ASTM F3269-17 RTA | RTCA DO-365B

---

## 1. System Losses (**L-1..N**)

- **L-1**: Loss of primary function.
- **L-2**: Loss of separation.

---

## 2. System Hazards (**H-1..N**)

- **H-1**: Trajectory excursion.

---

## 3. Hierarchical Control Structure Topology

The control structure consists of Controller directing Actuator.

---

## 4. Unsafe Control Actions (**UCA-1..N**)

{uca_table}

---

## 5. Loss Scenarios (**LS-1..N**) & Causal Factors

- **LS-1**: Signal loss drives containment breach.

---

## 6. Formal Safety Constraints (**SC-1..N**)

- **SC-1**: The system shall maintain state within envelope.

---

## 7. FMECA Criticality Matrix

{fmeca_table}

---

## 8. SORA SAIL Risk Mitigations & OSO Traceability Table

- **Ground Risk Class (GRC):** Assessed GRC = 4.
- **Air Risk Class (ARC):** Assessed ARC-c.
- **Specific Assurance and Integrity Level (SAIL):** SAIL III.

### Operational Safety Objectives (OSO-01 through OSO-24)

{oso_table}

---

## 9. ASTM F3269-17 Run-Time Assurance (RTA) & Commercial Toolchain Architecture

The safety net architecture complies with **ASTM F3269-17** and synthesizes into **MATLAB / Simulink / Stateflow / Embedded Coder** with SLDV formal invariant proofs.

---

## 10. Formal Safety Proof Suite

### Theorem THM-01: Invariant Preservation

1. **Proposition / Theorem Statement**: Bound on invariant.
2. **Operational Assumptions & Domain Bounds**: Bounded states.
3. **Invariant / Barrier Function Definition**: B(x) >= 0.
4. **Analytical / Inductive Derivation**: Derivation step.
5. **Formal Conclusion & Q.E.D.**: Preserved throughout operation. Q.E.D.
"""


class TestFMECAASTCoverageGate(unittest.TestCase):
    """Unit tests for AST-driven FMECA part def coverage, multiplicity, and integer RPN."""

    def test_rejection_of_fmeca_missing_ast_declared_part(self):
        """Verify FMECA table missing an AST-declared part def is rejected and names the missing part."""
        model = build_test_sysml_model(["FlightController", "NavigationSensor", "ActuatorUnit"], actions_per_part=2)
        # FMECA has FlightController and NavigationSensor, but misses ActuatorUnit
        comp_map = {
            "FlightController": [
                ("CPU Lockup", 4, 2, 2, 16, "SSOT"),
                ("RTOS Deadline Miss", 5, 2, 2, 20, "SSOT"),
                ("Flash CRC Corruption", 3, 3, 2, 18, "Derived"),
                ("Watchdog Reset Fail", 4, 2, 2, 16, "SSOT"),
                ("Stack Overflow", 4, 2, 2, 16, "Derived"),
                ("Interrupt Storm", 4, 2, 2, 16, "SSOT"),
                ("RAM Bit Flip", 3, 2, 2, 12, "Derived"),
                ("Clock Drift", 3, 2, 2, 12, "SSOT"),
            ],
            "NavigationSensor": [
                ("IMU Bias Drift", 4, 2, 2, 16, "SSOT"),
                ("GPS Satellite Loss", 3, 3, 2, 18, "Derived"),
                ("SPI Bus Timeout", 4, 2, 2, 16, "SSOT"),
                ("Barometer Stale Data", 3, 2, 2, 12, "Derived"),
                ("Magnetometer Distortion", 3, 2, 2, 12, "SSOT"),
                ("Data Outlier Spike", 3, 3, 2, 18, "Derived"),
                ("Sensor Brownout", 4, 2, 2, 16, "SSOT"),
                ("Packet CRC Failure", 3, 2, 2, 12, "Derived"),
            ],
        }
        doc = build_test_safety_document(comp_map, total_actions=6)
        errors = validate_safety_matrix_content(doc, model_text=model)

        self.assertTrue(
            any("FMECA table missing declared AST part def component(s): ActuatorUnit" in err for err in errors),
            f"Expected missing ActuatorUnit error, got: {errors}"
        )

    def test_rejection_of_fmeca_missing_universal_failure_dimension(self):
        """Verify FMECA table missing any universal failure dimension (e.g. Resource) is rejected."""
        model = build_test_sysml_model(["FlightController", "NavigationSensor"], actions_per_part=3)
        # comp_map has Interface, State, Action, but missing Resource
        comp_map = {
            "FlightController": [
                ("Interface Bus Timeout", 4, 2, 2, 16, "SSOT"),
                ("RTOS Deadline Miss", 5, 2, 2, 20, "SSOT"),
                ("Watchdog Reset Fail", 4, 2, 2, 16, "SSOT"),
            ],
            "NavigationSensor": [
                ("IMU Bias Drift", 4, 2, 2, 16, "SSOT"),
                ("GPS Satellite Loss", 3, 3, 2, 18, "Derived"),
                ("SPI Bus Timeout", 4, 2, 2, 16, "SSOT"),
            ],
        }
        doc = build_test_safety_document(comp_map, total_actions=6)
        errors = validate_safety_matrix_content(doc, model_text=model)

        self.assertTrue(
            any("missing coverage for universal failure dimension(s): Resource" in err for err in errors),
            f"Expected missing Resource dimension error, got: {errors}"
        )

    def test_rejection_of_high_criticality_part_missing_port_mode(self):
        """Verify high-criticality part (Crit >= 8) missing port-level failure mode for declared port is rejected."""
        sysml_text = """package TestSystem {
    part def Controller {
        port control_bus_in : PortTypeA;
        port telem_bus_out : PortTypeB;
        hazard FatalHazard {
            attribute severity : Integer = 9;
        }
        action def Action01;
        action def Action02;
        action def Action03;
        action def Action04;
    }
    requirement def Req_SC_01;
    requirement def Req_SC_02;
    requirement def Req_SC_03;
    requirement def Req_SC_04;
    requirement def Req_SC_05;
    requirement def Req_SC_06;
    requirement def Req_SC_07;
    requirement def Req_SC_08;
    requirement def Req_SC_09;
    requirement def Req_SC_10;
    requirement def Req_SC_11;
    requirement def Req_SC_12;
    requirement def Req_SC_13;
    requirement def Req_SC_14;
    requirement def Req_SC_15;
    requirement def Req_SC_16;
    requirement def Req_SC_17;
    requirement def Req_SC_18;
    requirement def Req_SC_19;
    requirement def Req_SC_20;
}
"""
        # Only covers control_bus_in, missing telem_bus_out
        comp_map = {
            "Controller": [
                ("control_bus_in Interface Packet Loss", 4, 2, 2, 16, "SSOT"),
                ("State Transition Freeze", 5, 1, 2, 10, "SSOT"),
                ("Command Rate Saturation", 3, 2, 2, 12, "Derived"),
                ("CPU Memory Overflow", 4, 2, 2, 16, "Derived"),
            ],
        }
        doc = build_test_safety_document(comp_map, total_actions=4)
        errors = validate_safety_matrix_content(doc, model_text=sysml_text)

        self.assertTrue(
            any("High-criticality component 'Controller' (Crit=9 >= 8) missing port-level interface failure mode for declared port(s): telem_bus_out" in err for err in errors),
            f"Expected missing telem_bus_out port error, got: {errors}"
        )

    def test_acceptance_of_complete_ast_anchored_multi_mode_fmeca(self):
        """Verify complete AST-anchored multi-mode FMECA table with Basis annotations passes."""
        model = build_test_sysml_model(["FlightController", "NavigationSensor", "ActuatorUnit"], actions_per_part=2)
        comp_map = {
            "FlightController": [
                ("CPU Lockup", 4, 2, 2, 16, "SSOT"),
                ("RTOS Deadline Miss", 5, 2, 2, 20, "SSOT"),
                ("Flash CRC Corruption", 3, 3, 2, 18, "Derived"),
                ("Watchdog Reset Fail", 4, 2, 2, 16, "SSOT"),
                ("Stack Overflow", 4, 2, 2, 16, "Derived"),
            ],
            "NavigationSensor": [
                ("IMU Bias Drift", 4, 2, 2, 16, "SSOT"),
                ("GPS Satellite Loss", 3, 3, 2, 18, "Derived"),
                ("SPI Bus Timeout", 4, 2, 2, 16, "SSOT"),
                ("Barometer Stale Data", 3, 2, 2, 12, "Derived"),
                ("Data Outlier Spike", 3, 3, 2, 18, "Derived"),
            ],
            "ActuatorUnit": [
                ("CAN Bus Babbling Idiot", 4, 2, 2, 16, "SSOT"),
                ("Torque Saturation", 3, 2, 3, 18, "Derived"),
                ("PWM Jitter", 3, 2, 2, 12, "SSOT"),
                ("Thermal Overheat", 4, 2, 2, 16, "SSOT"),
                ("Encoder Stalls", 4, 2, 2, 16, "Derived"),
            ],
        }
        doc = build_test_safety_document(comp_map, total_actions=6)
        errors = validate_safety_matrix_content(doc, model_text=model)

        self.assertEqual(errors, [], f"Expected 0 errors for complete FMECA table, got: {errors}")

    def test_rejection_of_invalid_rpn_calculation(self):
        """Verify FMECA table with invalid RPN calculation (RPN != S * O * D) is rejected."""
        comp_map = {
            "FlightController": [
                ("CPU Lockup", 4, 2, 2, 25, "SSOT"),  # Invalid: 4*2*2 = 16, but RPN is 25
                ("RTOS Deadline Miss", 5, 2, 2, 20, "SSOT"),
                ("Flash CRC Corruption", 3, 3, 2, 18, "Derived"),
            ],
            "NavigationSensor": [
                ("IMU Bias Drift", 4, 2, 2, 16, "SSOT"),
                ("GPS Satellite Loss", 3, 3, 2, 18, "Derived"),
                ("SPI Bus Timeout", 4, 2, 2, 16, "SSOT"),
            ],
            "ActuatorUnit": [
                ("CAN Bus Babbling Idiot", 4, 2, 2, 16, "SSOT"),
                ("Torque Saturation", 3, 2, 3, 18, "Derived"),
                ("PWM Jitter", 3, 2, 2, 12, "SSOT"),
                ("Thermal Overheat", 4, 2, 2, 16, "SSOT"),
                ("Encoder Stalls", 4, 2, 2, 16, "Derived"),
                ("Motor Windings Short", 5, 1, 2, 10, "SSOT"),
                ("Bearing Seize", 4, 1, 2, 8, "Derived"),
                ("Voltage Sag", 4, 2, 2, 16, "SSOT"),
                ("Ground Fault", 4, 2, 2, 16, "Derived"),
            ],
        }
        doc = build_test_safety_document(comp_map, total_actions=6)
        errors = validate_safety_matrix_content(doc)

        self.assertTrue(
            any("invalid RPN calculation -- expected S(4) * O(2) * D(2) = 16, but found RPN = 25" in err for err in errors),
            f"Expected invalid RPN error, got: {errors}"
        )

    def test_rejection_of_fmeca_with_undeclared_phantom_component(self):
        """Verify FMECA table referencing undeclared phantom component not in AST is rejected (Issue #251, #249)."""
        model = build_test_sysml_model(["FlightController", "NavigationSensor"], actions_per_part=3)
        comp_map = {
            "FlightController": [
                ("CPU Lockup", 4, 2, 2, 16, "SSOT"),
                ("RTOS Deadline Miss", 5, 2, 2, 20, "SSOT"),
                ("Flash CRC Corruption", 3, 3, 2, 18, "Derived"),
                ("Watchdog Reset Fail", 4, 2, 2, 16, "SSOT"),
            ],
            "NavigationSensor": [
                ("IMU Bias Drift", 4, 2, 2, 16, "SSOT"),
                ("GPS Satellite Loss", 3, 3, 2, 18, "Derived"),
                ("SPI Bus Timeout", 4, 2, 2, 16, "SSOT"),
            ],
            "PhantomSubsystem": [
                ("Ghost Logic Fail", 4, 2, 2, 16, "SSOT"),
            ],
        }
        doc = build_test_safety_document(comp_map, total_actions=6)
        errors, report, _ = validate_safety_matrix_ast(doc, model_text=model)

        self.assertIn("PhantomSubsystem", report.undeclared_fmeca_parts)
        self.assertTrue(
            any("FMECA table references undeclared phantom component(s) not in AST: PhantomSubsystem" in err for err in errors),
            f"Expected undeclared phantom component error, got: {errors}"
        )

    def test_bidirectional_ast_closure_reports_both_missing_and_undeclared(self):
        """Verify AST verification reports both missing declared parts and extra phantom parts simultaneously."""
        model = build_test_sysml_model(["FlightController", "NavigationSensor", "ActuatorUnit"], actions_per_part=2)
        # Misses ActuatorUnit, includes PhantomUnit
        comp_map = {
            "FlightController": [
                ("CPU Lockup", 4, 2, 2, 16, "SSOT"),
                ("RTOS Deadline Miss", 5, 2, 2, 20, "SSOT"),
                ("Flash CRC Corruption", 3, 3, 2, 18, "Derived"),
                ("Watchdog Reset Fail", 4, 2, 2, 16, "SSOT"),
            ],
            "NavigationSensor": [
                ("IMU Bias Drift", 4, 2, 2, 16, "SSOT"),
                ("GPS Satellite Loss", 3, 3, 2, 18, "Derived"),
                ("SPI Bus Timeout", 4, 2, 2, 16, "SSOT"),
            ],
            "PhantomUnit": [
                ("Phantom Fault", 4, 2, 2, 16, "SSOT"),
            ],
        }
        doc = build_test_safety_document(comp_map, total_actions=6)
        errors, report, _ = validate_safety_matrix_ast(doc, model_text=model)

        self.assertIn("ActuatorUnit", report.missing_fmeca_parts)
        self.assertIn("PhantomUnit", report.undeclared_fmeca_parts)
        self.assertTrue(any("missing declared AST part def component(s): ActuatorUnit" in err for err in errors))
        self.assertTrue(any("references undeclared phantom component(s) not in AST: PhantomUnit" in err for err in errors))

    def test_format_cli_summary_includes_undeclared_parts(self):
        """Verify format_cli_summary includes undeclared FMECA parts count."""
        model = build_test_sysml_model(["FlightController"], actions_per_part=2)
        comp_map = {
            "FlightController": [
                ("CPU Lockup", 4, 2, 2, 16, "SSOT"),
                ("RTOS Deadline Miss", 5, 2, 2, 20, "SSOT"),
                ("Flash CRC Corruption", 3, 3, 2, 18, "Derived"),
                ("Watchdog Reset Fail", 4, 2, 2, 16, "SSOT"),
            ],
            "PhantomA": [("Fault A", 4, 2, 2, 16, "SSOT")],
            "PhantomB": [("Fault B", 4, 2, 2, 16, "SSOT")],
        }
        doc = build_test_safety_document(comp_map, total_actions=2)
        _, report, _ = validate_safety_matrix_ast(doc, model_text=model)
        cli_summary = report.format_cli_summary()
        self.assertIn("2 undeclared FMECA part(s)", cli_summary)


def test_end_to_end_check17_ast_fmeca_missing_part_fails(tmp_path):
    """Verify end-to-end check_safety_integrity_and_sora_completeness fails when AST part is missing from FMECA."""
    model = build_test_sysml_model(["FlightController", "NavigationSensor", "ActuatorUnit"], actions_per_part=2)
    schema_dir = tmp_path / "schema"
    schema_dir.mkdir(parents=True, exist_ok=True)
    (schema_dir / "model.sysml").write_text(model, encoding="utf-8")

    comp_map = {
        "FlightController": [
            (f"Mode {i}", 4, 2, 2, 16, "SSOT") for i in range(1, 10)
        ],
        "NavigationSensor": [
            (f"Mode {i}", 4, 2, 2, 16, "SSOT") for i in range(1, 10)
        ],
    }
    doc = build_test_safety_document(comp_map, total_actions=6)
    safety_dir = tmp_path / "docs" / "safety"
    safety_dir.mkdir(parents=True, exist_ok=True)
    (safety_dir / "STPA_MATRIX.md").write_text(doc, encoding="utf-8")

    with pytest.raises(SystemExit) as exc_info:
        check_safety_integrity_and_sora_completeness(str(tmp_path))
    assert exc_info.value.code == 1


def test_end_to_end_check17_ast_fmeca_undeclared_phantom_part_fails(tmp_path):
    """Verify end-to-end check_safety_integrity_and_sora_completeness fails when FMECA references an undeclared phantom part."""
    model = build_test_sysml_model(["FlightController", "NavigationSensor"], actions_per_part=2)
    schema_dir = tmp_path / "schema"
    schema_dir.mkdir(parents=True, exist_ok=True)
    (schema_dir / "model.sysml").write_text(model, encoding="utf-8")

    comp_map = {
        "FlightController": [
            (f"Mode {i}", 4, 2, 2, 16, "SSOT") for i in range(1, 5)
        ],
        "NavigationSensor": [
            (f"Mode {i}", 4, 2, 2, 16, "SSOT") for i in range(1, 5)
        ],
        "PhantomSubsystem": [
            ("Ghost Failure", 4, 2, 2, 16, "SSOT"),
        ],
    }
    doc = build_test_safety_document(comp_map, total_actions=4)
    safety_dir = tmp_path / "docs" / "safety"
    safety_dir.mkdir(parents=True, exist_ok=True)
    (safety_dir / "STPA_MATRIX.md").write_text(doc, encoding="utf-8")

    with pytest.raises(SystemExit) as exc_info:
        check_safety_integrity_and_sora_completeness(str(tmp_path))
    assert exc_info.value.code == 1

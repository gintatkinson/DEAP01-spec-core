"""
Topological Criticality Calculation, 4-Dimensional Failure Mode Coverage,
and 16-Column FMECA AST Closure Verification Suite.
/// Realises: [Feat-070/TopologicalCriticalityClosure, FMECA_4D_Dimensions, HighCriticalityPortCoverage]
"""
import os
import sys
import unittest
import pytest

repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if repo_root not in sys.path:
    sys.path.insert(0, repo_root)

# Ensure spec-orchestrator scripts are on sys.path
spec_scripts_dir = os.path.join(repo_root, "skills", "spec-orchestrator", "scripts")
if spec_scripts_dir not in sys.path:
    sys.path.insert(0, spec_scripts_dir)

from sysmlv2_ast import (
    SysMLParser,
    SysMLPackage,
    PartDef,
    PortDef,
    HazardDef,
    ConnectionDef,
)

from scripts.verify_downstream_baseline import (
    calculate_topological_criticality,
    check_failure_dimension_coverage,
    check_high_criticality_port_coverage,
    check_fmeca_ast_coverage,
    parse_fmeca_table,
    validate_safety_matrix_content,
    validate_safety_matrix_ast,
    check_safety_integrity_and_sora_completeness,
    UNIVERSAL_FAILURE_DIMENSIONS,
)


def build_neutral_safety_matrix_document(
    fmeca_table_text: str,
    total_actions: int = 4,
    oso_count: int = 24,
) -> str:
    """Build a complete neutral 8-pillar safety matrix document wrapping a custom FMECA table."""
    guide_words = [
        "Not providing causes hazard",
        "Providing causes hazard",
        "Providing too early, too late, or out of order",
        "Stopped too soon or applied too long",
    ]
    uca_rows = []
    uca_idx = 1
    for act_idx in range(1, total_actions + 1):
        for gw in guide_words:
            uca_rows.append(
                f"| UCA-{uca_idx:02d} | ControllerNode | Action_{act_idx:02d} | {gw} | H-1 | L-1 | SC-{uca_idx:02d} |"
            )
            uca_idx += 1

    uca_table = (
        "| UCA ID | Controller | Control Action | Guide Word | Hazard Ref | System Loss Ref | Safety Constraint |\n"
        "| :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n"
        + "\n".join(uca_rows)
    )

    oso_table = (
        "| OSO ID | Robustness Level | Justification | Mitigation Ref |\n"
        "| :--- | :--- | :--- | :--- |\n"
        + "\n".join(
            f"| OSO-{i:02d} | Robust | Compliance verification for objective {i:02d} | M-{i:02d} |"
            for i in range(1, oso_count + 1)
        )
    )

    return rf"""# STPA Safety Analysis, FMECA Matrix & SORA SAIL Assessment

> **Primary Commercial Toolchain Integration Context:** MATLAB / Simulink / Stateflow / Embedded Coder
> **Safety Standards:** JARUS SORA v2.5 | ASTM F3269-17 RTA | RTCA DO-365B

---

## 1. System Losses (**L-1..N**)

- **L-1**: Loss of primary system function.
- **L-2**: Loss of secondary boundary control.

---

## 2. System Hazards (**H-1..N**)

- **H-1**: Boundary state excursion.

---

## 3. Hierarchical Control Structure Topology

The system comprises ControllerNode directing ActuatorNode.

---

## 4. Unsafe Control Actions (**UCA-1..N**)

{uca_table}

---

## 5. Loss Scenarios (**LS-1..N**) & Causal Factors

- **LS-1**: Signal loss drives boundary containment breach.

---

## 6. Formal Safety Constraints (**SC-1..N**)

- **SC-1**: The system shall maintain state within certified operational envelope.

---

## 7. FMECA Criticality Matrix

{fmeca_table_text}

---

## 8. SORA SAIL Risk Mitigations & OSO Traceability Table

- **Ground Risk Class (GRC):** Final GRC = 4.
- **Air Risk Class (ARC):** Final ARC-c.
- **Specific Assurance and Integrity Level (SAIL):** SAIL III.

### Operational Safety Objectives (OSO-01 through OSO-24)

{oso_table}

---

## 9. ASTM F3269-17 Run-Time Assurance (RTA) & Commercial Toolchain Architecture

The safety monitor complies with **ASTM F3269-17** and synthesizes to **MATLAB / Simulink / Stateflow / Embedded Coder**.

---

## 10. Formal Safety Proof Suite

### Theorem THM-01: Invariant Preservation

1. **Proposition / Theorem Statement**: System state remains forward invariant.
2. **Operational Assumptions & Domain Bounds**: Bounded disturbance envelope.
3. **Invariant / Barrier Function Definition**: B(x) >= 0.
4. **Analytical / Inductive Derivation**: Lie derivative calculation satisfies Nagumo condition.
5. **Formal Conclusion & Q.E.D.**: Invariant holds for all t >= 0. Q.E.D.
"""


class TestTopologicalCriticalityCalculation(unittest.TestCase):
    """Unit tests for dynamic topological criticality calculation on SysML v2 AST."""

    def test_direct_part_hazard_criticality(self):
        """Verify part with direct attached hazard receives its maximum hazard severity."""
        sysml_text = """
        package SystemModel {
            part def ProcessingUnit {
                hazard HazSevere {
                    attribute severity : Integer = 9;
                }
            }
        }
        """
        pkg = SysMLParser.parse_text(sysml_text)
        part = pkg.find_part("ProcessingUnit")
        self.assertIsNotNone(part)
        crit = calculate_topological_criticality(pkg, part)
        self.assertEqual(crit, 9)

    def test_connected_port_hazard_propagation(self):
        """Verify hazard severity propagates across port-to-port topological connections."""
        sysml_text = """
        package SystemModel {
            part def SenderUnit {
                port out_data : PortTypeA;
            }
            part def ReceiverUnit {
                port in_data : PortTypeB;
                hazard HazReceiver {
                    attribute severity : Integer = 8;
                    attribute source_port : String = "ReceiverUnit.in_data";
                }
            }
            connection Link1 {
                connect SenderUnit.out_data to ReceiverUnit.in_data;
            }
        }
        """
        pkg = SysMLParser.parse_text(sysml_text)
        sender = pkg.find_part("SenderUnit")
        receiver = pkg.find_part("ReceiverUnit")
        self.assertIsNotNone(sender)
        self.assertIsNotNone(receiver)

        crit_receiver = calculate_topological_criticality(pkg, receiver)
        self.assertEqual(crit_receiver, 8)

        crit_sender = calculate_topological_criticality(pkg, sender)
        self.assertEqual(crit_sender, 8)

    def test_multi_hop_topological_propagation(self):
        """Verify topological criticality propagates across multi-hop connections."""
        sysml_text = """
        package SystemModel {
            part def NodeA {
                port pA : PortA;
            }
            part def NodeB {
                port pB_in : PortB;
                port pB_out : PortB;
            }
            part def NodeC {
                port pC_in : PortC;
                hazard CriticalHazard {
                    attribute severity : Integer = 10;
                    attribute target_port : String = "NodeC.pC_in";
                }
            }
            connection ConnAB {
                connect NodeA.pA to NodeB.pB_in;
            }
            connection ConnBC {
                connect NodeB.pB_out to NodeC.pC_in;
            }
        }
        """
        pkg = SysMLParser.parse_text(sysml_text)
        node_a = pkg.find_part("NodeA")
        node_b = pkg.find_part("NodeB")
        node_c = pkg.find_part("NodeC")

        self.assertEqual(calculate_topological_criticality(pkg, node_c), 10)
        self.assertEqual(calculate_topological_criticality(pkg, node_b), 10)
        self.assertEqual(calculate_topological_criticality(pkg, node_a), 10)

    def test_default_base_severity_unattached(self):
        """Verify part with no hazards attached or reachable defaults to base severity 1."""
        sysml_text = """
        package SystemModel {
            part def IsolatedNode {
                port status_port : PortA;
            }
        }
        """
        pkg = SysMLParser.parse_text(sysml_text)
        part = pkg.find_part("IsolatedNode")
        self.assertIsNotNone(part)
        crit = calculate_topological_criticality(pkg, part)
        self.assertEqual(crit, 1)


class TestUniversalFailureDimensions(unittest.TestCase):
    """Unit tests for the 4 Universal Failure Dimensions (Interface, State, Action, Resource)."""

    def test_four_dimensions_acceptance(self):
        """Verify FMECA table covering Interface (Γ), State (Φ), Action (Ω), and Resource (Ψ) passes."""
        table = """
| Failure ID | Component | Failure Mode | Local Effect | System Effect | S | O | D | RPN | Mitigating Design Control | Basis |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| FM-01 | ControllerNode | Interface Bus Timeout | Comms delay | L-1 | 4 | 2 | 2 | 16 | Redundant Bus | [SSOT: Interface] |
| FM-02 | ControllerNode | Statechart Transition Deadlock | Logic freeze | L-1 | 5 | 1 | 2 | 10 | Watchdog Reset | [SSOT: State] |
| FM-03 | ControllerNode | Command Execution Latency Jitter | Timing slip | L-2 | 3 | 2 | 2 | 12 | Deterministic Loop | [Derived: Action] |
| FM-04 | ControllerNode | Memory Buffer Overflow | Frame drop | L-1 | 4 | 2 | 2 | 16 | Buffer Pool Bound | [Derived: Resource] |
        """
        fmeca_data = parse_fmeca_table(table)
        missing = check_failure_dimension_coverage(fmeca_data)
        self.assertEqual(missing, [], f"Expected 0 missing dimensions, got: {missing}")

    def test_missing_dimension_rejection(self):
        """Verify FMECA table missing Resource dimension reports violation naming Resource."""
        table = """
| Failure ID | Component | Failure Mode | Local Effect | System Effect | S | O | D | RPN | Mitigating Design Control | Basis |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| FM-01 | ControllerNode | Interface Bus Timeout | Comms delay | L-1 | 4 | 2 | 2 | 16 | Redundant Bus | [SSOT: Γ] |
| FM-02 | ControllerNode | Statechart Deadlock | Logic freeze | L-1 | 5 | 1 | 2 | 10 | Watchdog Reset | [SSOT: Φ] |
| FM-03 | ControllerNode | Command Execution Latency Jitter | Timing slip | L-2 | 3 | 2 | 2 | 12 | Deterministic Loop | [Derived: Ω] |
        """
        fmeca_data = parse_fmeca_table(table)
        missing = check_failure_dimension_coverage(fmeca_data)
        self.assertIn("Resource", missing)

        doc = build_neutral_safety_matrix_document(table)
        errors = validate_safety_matrix_content(doc)
        self.assertTrue(
            any("missing coverage for universal failure dimension(s): Resource" in err for err in errors),
            f"Expected Resource missing error, got: {errors}"
        )


class TestHighCriticalityPortCoverage(unittest.TestCase):
    """Unit tests for port-level interface failure mode enforcement on high-criticality parts (Crit >= 8)."""

    def test_high_criticality_part_missing_port_mode_rejected(self):
        """Verify high-criticality part (Crit=9 >= 8) missing port-level failure mode for declared port is rejected."""
        sysml_text = """
        package SystemModel {
            part def CoreController {
                port cmd_in : CommandPort;
                port telem_out : TelemPort;
                hazard HazFatal {
                    attribute severity : Integer = 9;
                }
                action def Action_01;
                action def Action_02;
                action def Action_03;
                action def Action_04;
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
        # FMECA only covers cmd_in, but misses telem_out
        table = """
| Failure ID | Component | Failure Mode | Local Effect | System Effect | S | O | D | RPN | Mitigating Design Control | Basis |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| FM-01 | CoreController | cmd_in Interface Packet Loss | Comms delay | L-1 | 4 | 2 | 2 | 16 | Redundant Channel | [SSOT: Interface] |
| FM-02 | CoreController | State Transition Freeze | Logic stall | L-1 | 5 | 1 | 2 | 10 | Watchdog Supervisor | [SSOT: State] |
| FM-03 | CoreController | Command Rate Clamping | Actuation limit | L-2 | 3 | 2 | 2 | 12 | Rate Limiter | [Derived: Action] |
| FM-04 | CoreController | CPU Memory Buffer Exhaustion | Heap overflow | L-1 | 4 | 2 | 2 | 16 | Static Allocation | [Derived: Resource] |
        """
        doc = build_neutral_safety_matrix_document(table)
        doc = doc.replace("ControllerNode", "CoreController")
        errors = validate_safety_matrix_content(doc, model_text=sysml_text)

        self.assertTrue(
            any("High-criticality component 'CoreController' (Crit=9 >= 8) missing port-level interface failure mode for declared port(s): telem_out" in err for err in errors),
            f"Expected missing telem_out port error, got: {errors}"
        )

    def test_high_criticality_part_all_ports_covered_accepted(self):
        """Verify high-criticality part (Crit=9 >= 8) with all declared ports covered in FMECA is accepted."""
        sysml_text = """
        package SystemModel {
            part def CoreController {
                port cmd_in : CommandPort;
                port telem_out : TelemPort;
                hazard HazFatal {
                    attribute severity : Integer = 9;
                }
                action def Action_01;
                action def Action_02;
                action def Action_03;
                action def Action_04;
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
        table = """
| Failure ID | Component | Failure Mode | Local Effect | System Effect | S | O | D | RPN | Mitigating Design Control | Basis |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| FM-01 | CoreController | cmd_in Interface Packet Loss | Comms delay | L-1 | 4 | 2 | 2 | 16 | Redundant Channel | [SSOT: Interface] |
| FM-02 | CoreController | telem_out Bus Frame Timeout | Telemetry drop | L-2 | 3 | 2 | 2 | 12 | Retransmission Queue | [SSOT: Interface] |
| FM-03 | CoreController | Statechart Sync Desync | Mode disagreement | L-1 | 5 | 1 | 2 | 10 | Watchdog Supervisor | [SSOT: State] |
| FM-04 | CoreController | Command Rate Saturation | Authority limit | L-2 | 3 | 2 | 2 | 12 | Rate Limiter | [Derived: Action] |
| FM-05 | CoreController | CPU Memory Buffer Overflow | Frame drop | L-1 | 4 | 2 | 2 | 16 | Static Allocation | [Derived: Resource] |
        """
        doc = build_neutral_safety_matrix_document(table)
        doc = doc.replace("ControllerNode", "CoreController")
        errors = validate_safety_matrix_content(doc, model_text=sysml_text)
        self.assertEqual(errors, [], f"Expected 0 errors, got: {errors}")

    def test_low_criticality_part_unconstrained_ports_accepted(self):
        """Verify low-criticality part (Crit=4 < 8) is not rejected if individual ports are not explicitly named."""
        sysml_text = """
        package SystemModel {
            part def SecondarySensor {
                port aux_in : AuxPort;
                port aux_out : AuxPort;
                hazard HazMinor {
                    attribute severity : Integer = 4;
                }
                action def Action_01;
                action def Action_02;
                action def Action_03;
                action def Action_04;
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
        table = """
| Failure ID | Component | Failure Mode | Local Effect | System Effect | S | O | D | RPN | Mitigating Design Control | Basis |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| FM-01 | SecondarySensor | Interface Bus Timeout | Comms delay | L-1 | 4 | 2 | 2 | 16 | Redundant Channel | [SSOT: Interface] |
| FM-02 | SecondarySensor | State Bias Drift | Measurement offset | L-1 | 3 | 2 | 2 | 12 | Calibration Offset | [SSOT: State] |
| FM-03 | SecondarySensor | Output Timing Jitter | Timing slip | L-2 | 3 | 2 | 2 | 12 | Low-Pass Filter | [Derived: Action] |
| FM-04 | SecondarySensor | Power Voltage Sag | Brownout reset | L-1 | 4 | 2 | 2 | 16 | Decoupling Capacitors | [Derived: Resource] |
        """
        doc = build_neutral_safety_matrix_document(table)
        doc = doc.replace("ControllerNode", "SecondarySensor")
        errors = validate_safety_matrix_content(doc, model_text=sysml_text)
        self.assertEqual(errors, [], f"Expected 0 errors for low-criticality part, got: {errors}")


class TestStructural16ColumnTableData(unittest.TestCase):
    """Unit tests for structural 16-column table parsing, RPN arithmetic, and Basis tags."""

    def test_structural_16_column_table_valid(self):
        """Verify full 16-column FMECA table with valid SOD integer ratings [1, 10], RPN, and Basis tags passes."""
        table = """
| Failure ID | Component | Failure Mode | Failure Dimension | Port Ref | Local Effect | System Effect | S | O | D | RPN | Mitigating Design Control | Detection Method | Severity Class | Action Item | Basis |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| FM-01 | ControllerA | Port Bus Timeout | Interface | port_rx | Loss of frame | L-1 | 5 | 2 | 2 | 20 | Dual Channel | Heartbeat BIT | Critical | ACT-01 | [SSOT: AST:ControllerA] |
| FM-02 | ControllerA | State Deadlock | State | None | Task freeze | L-1 | 6 | 1 | 2 | 12 | Hardware Watchdog | Watchdog Pulse | Critical | ACT-02 | [SSOT: AST:ControllerA] |
| FM-03 | ControllerA | Command Jitter | Action | port_tx | Execution slip | L-2 | 4 | 2 | 2 | 16 | Rate Limiter | Timestamp Delta | Moderate | ACT-03 | [Derived: UCA-01] |
| FM-04 | ControllerA | Memory Buffer Overflow | Resource | None | Data corrupt | L-1 | 4 | 2 | 2 | 16 | Ring Buffer Pool | High-Water Mark | Moderate | ACT-04 | [Derived: UCA-02] |
        """
        doc = build_neutral_safety_matrix_document(table)
        doc = doc.replace("ControllerNode", "ControllerA")
        errors = validate_safety_matrix_content(doc)
        self.assertEqual(errors, [], f"Expected 0 errors for valid 16-column table, got: {errors}")

    def test_structural_16_column_table_invalid_rpn_rejected(self):
        """Verify 16-column table with incorrect RPN math is rejected."""
        table = """
| Failure ID | Component | Failure Mode | Failure Dimension | Port Ref | Local Effect | System Effect | S | O | D | RPN | Mitigating Design Control | Detection Method | Severity Class | Action Item | Basis |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| FM-01 | ControllerA | Port Bus Timeout | Interface | port_rx | Loss of frame | L-1 | 5 | 2 | 2 | 99 | Dual Channel | Heartbeat BIT | Critical | ACT-01 | [SSOT: AST:ControllerA] |
| FM-02 | ControllerA | State Deadlock | State | None | Task freeze | L-1 | 6 | 1 | 2 | 12 | Hardware Watchdog | Watchdog Pulse | Critical | ACT-02 | [SSOT: AST:ControllerA] |
| FM-03 | ControllerA | Command Jitter | Action | port_tx | Execution slip | L-2 | 4 | 2 | 2 | 16 | Rate Limiter | Timestamp Delta | Moderate | ACT-03 | [Derived: UCA-01] |
| FM-04 | ControllerA | Memory Buffer Overflow | Resource | None | Data corrupt | L-1 | 4 | 2 | 2 | 16 | Ring Buffer Pool | High-Water Mark | Moderate | ACT-04 | [Derived: UCA-02] |
        """
        doc = build_neutral_safety_matrix_document(table)
        errors = validate_safety_matrix_content(doc)
        self.assertTrue(
            any("expected S(5) * O(2) * D(2) = 20, but found RPN = 99" in err for err in errors),
            f"Expected RPN error, got: {errors}"
        )

    def test_structural_16_column_table_out_of_range_ratings_rejected(self):
        """Verify 16-column table with S/O/D ratings outside [1, 10] is rejected."""
        table = """
| Failure ID | Component | Failure Mode | Failure Dimension | Port Ref | Local Effect | System Effect | S | O | D | RPN | Mitigating Design Control | Detection Method | Severity Class | Action Item | Basis |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| FM-01 | ControllerA | Port Bus Timeout | Interface | port_rx | Loss of frame | L-1 | 12 | 2 | 2 | 48 | Dual Channel | Heartbeat BIT | Critical | ACT-01 | [SSOT: AST:ControllerA] |
| FM-02 | ControllerA | State Deadlock | State | None | Task freeze | L-1 | 6 | 1 | 2 | 12 | Hardware Watchdog | Watchdog Pulse | Critical | ACT-02 | [SSOT: AST:ControllerA] |
| FM-03 | ControllerA | Command Jitter | Action | port_tx | Execution slip | L-2 | 4 | 2 | 2 | 16 | Rate Limiter | Timestamp Delta | Moderate | ACT-03 | [Derived: UCA-01] |
| FM-04 | ControllerA | Memory Buffer Overflow | Resource | None | Data corrupt | L-1 | 4 | 2 | 2 | 16 | Ring Buffer Pool | High-Water Mark | Moderate | ACT-04 | [Derived: UCA-02] |
        """
        doc = build_neutral_safety_matrix_document(table)
        errors = validate_safety_matrix_content(doc)
        self.assertTrue(
            any("ratings out of range [1, 10] (S=12, O=2, D=2)" in err for err in errors),
            f"Expected out of range error, got: {errors}"
        )

    def test_structural_16_column_table_missing_basis_rejected(self):
        """Verify 16-column table missing explicit Derivation Basis is rejected."""
        table = """
| Failure ID | Component | Failure Mode | Failure Dimension | Port Ref | Local Effect | System Effect | S | O | D | RPN | Mitigating Design Control | Detection Method | Severity Class | Action Item | Basis |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| FM-01 | ControllerA | Port Bus Timeout | Interface | port_rx | Loss of frame | L-1 | 5 | 2 | 2 | 20 | Dual Channel | Heartbeat BIT | Critical | ACT-01 | Unspecified |
| FM-02 | ControllerA | State Deadlock | State | None | Task freeze | L-1 | 6 | 1 | 2 | 12 | Hardware Watchdog | Watchdog Pulse | Critical | ACT-02 | Unspecified |
| FM-03 | ControllerA | Command Jitter | Action | port_tx | Execution slip | L-2 | 4 | 2 | 2 | 16 | Rate Limiter | Timestamp Delta | Moderate | ACT-03 | Unspecified |
| FM-04 | ControllerA | Memory Buffer Overflow | Resource | None | Data corrupt | L-1 | 4 | 2 | 2 | 16 | Ring Buffer Pool | High-Water Mark | Moderate | ACT-04 | Unspecified |
        """
        doc = build_neutral_safety_matrix_document(table)
        errors = validate_safety_matrix_content(doc)
        self.assertTrue(
            any("missing explicit Derivation Basis classification ('SSOT' / 'Derived')" in err for err in errors),
            f"Expected missing Basis error, got: {errors}"
        )


if __name__ == "__main__":
    unittest.main()

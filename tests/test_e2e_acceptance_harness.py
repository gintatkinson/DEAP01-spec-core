#!/usr/bin/env python3
"""
Test suite for the End-to-End Acceptance Harness (scripts/e2e_acceptance_harness.py).
Validates each of the 6 verification layers, 5 deterministic semantic solvers, and report generation.
"""

import json
import os
import shutil
import sys
import tempfile
import unittest

repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if repo_root not in sys.path:
    sys.path.insert(0, repo_root)

# Import harness components and solvers
from scripts.e2e_acceptance_harness import (
    AcceptanceHarness,
    LayerResult,
    DomainScorecard,
    HarnessSummary,
    POSITIVE_DOMAIN_LEXICONS,
    _infer_lexicon_domain_type,
    solve_relational_mass_cross_sum,
    solve_closed_form_quadratic_physics,
    solve_dimensional_energy_conservation,
    solve_normative_standards_cross_check,
    solve_forbidden_cross_domain_ontology,
    solve_positive_domain_lexicon_floor,
    extract_substantive_sentences,
    solve_pairwise_domain_similarity,
    verify_layer1_delivery_gate,
    verify_layer2_syntax_purity,
    verify_layer3_cardinality,
    verify_layer4_physical_math,
    verify_layer5_adversarial_invariants,
    verify_layer6_baseline_parity,
    evaluate_domain_workspace,
    generate_markdown_report,
    generate_json_scorecard,
)


class TestE2EAcceptanceHarnessLayers(unittest.TestCase):
    def setUp(self):
        self.temp_dir = tempfile.mkdtemp(prefix="test_harness_")
        self.docs_conops = os.path.join(self.temp_dir, "docs", "conops")
        os.makedirs(self.docs_conops, exist_ok=True)
        
        # Create minimal conforming CONOPS.md (>=800 lines with 12 sections)
        self.conops_lines = []
        self.conops_lines.append("| Attribute | Value |")
        self.conops_lines.append("| :--- | :--- |")
        self.conops_lines.append("| **Title** | Concept of Operations: Tactical ISR UAV |")
        self.conops_lines.append("")
        self.conops_lines.append("# Concept of Operations")
        self.conops_lines.append("")
        for i in range(1, 13):
            self.conops_lines.append(f"## {i}. Section {i} Title")
            self.conops_lines.append("")
            # Add 70 substantive lines per section
            for j in range(1, 71):
                self.conops_lines.append(f"Section {i} substantive description line {j} detailing operational concept.")
            self.conops_lines.append("")
        
        # Section 1.3.2 Table (Mass Partition Rows)
        self.conops_lines.append("### 1.3.2 Parametric Subsystem Mass/Resource Budget Breakdown Table")
        self.conops_lines.append("| Structural Group (AST Partition) | Allocated Subsystems & Components | Mass Fraction (% MTOW) | Mass Budget (kg) | Nominal Power Budget (W) | Peak Power Budget (W) |")
        self.conops_lines.append("| :--- | :--- | :--- | :--- | :--- | :--- |")
        self.conops_lines.append("| **Airframe Structure** | Fuselage primary structure | 34.0% | 8.50 | 0.0 | 0.0 |")
        self.conops_lines.append("| **Avionics & Processing** | Flight control computer | 12.8% | 3.20 | 45.0 | 75.0 |")
        self.conops_lines.append("| **Propulsion & Power Distribution** | Actuators and electric motors | 24.0% | 6.00 | 300.0 | 500.0 |")
        self.conops_lines.append("| **Energy Storage Subsystem** | Smart battery module | 18.0% | 4.50 | 0.0 | 0.0 |")
        self.conops_lines.append("| **Primary Mission Payload** | Multi-modal mission sensor suite | 8.0% | 2.00 | 45.0 | 80.0 |")
        self.conops_lines.append("| **Autonomous Failsafe Containment** | Independent safety watchdog | 3.2% | 0.80 | 10.0 | 25.0 |")
        self.conops_lines.append("| **Total System Integration** | **Integrated Cyber-Physical Platform** | **100.0% MTOW** | **25.0** | **400.0** | **680.0** |")
        self.conops_lines.append("")

        # Section 1.3.3 Table (Physical Limits)
        self.conops_lines.append("### 1.3.3 Master Physical Limits Table")
        self.conops_lines.append("| Parameter ID | Bounding Parameter Name | Parametric Symbol | Threshold (Boundary Limit) | Objective (Nominal Target) | Engineering Unit | Normative / Safety Basis |")
        self.conops_lines.append("| :--- | :--- | :--- | :--- | :--- | :--- | :--- |")
        self.conops_lines.append("| **PL-01** | Maximum Takeoff Weight (MTOW) | m_MTOW | <= 25.0 | 25.0 | kg | Certified limit |")
        self.conops_lines.append("| **PL-09** | Mission Operational Endurance | t_endurance | >= 4.0 | 4.0 | hours | Mission endurance |")
        self.conops_lines.append("")

        # Section 1.5 Normative Standards Table
        self.conops_lines.append("### 1.5 Normative Standards & Regulatory Baseline")
        self.conops_lines.append("| Standard ID | Issuing Body | Title & Baseline Edition | Applicable Clauses & Focus Areas |")
        self.conops_lines.append("| :--- | :--- | :--- | :--- |")
        self.conops_lines.append("| RTCA DO-178C | RTCA | Software Considerations | Safety Verification |")
        self.conops_lines.append("| RTCA DO-254 | RTCA | Electronic Hardware | Hardware Assurance |")
        self.conops_lines.append("| MIL-STD-882E | DoD | System Safety | Hazard Tracking |")
        self.conops_lines.append("| MIL-STD-810H | DoD | Environmental Engineering | Environmental testing |")
        self.conops_lines.append("| NIST SP 800-82r3 | NIST | OT Security | Anti-replay telemetry |")
        self.conops_lines.append("| ASTM F3411-22a | ASTM | Remote ID | Remote ID broadcast |")
        self.conops_lines.append("| JARUS SORA v2.5 | JARUS | Risk Assessment | SAIL and GRC |")
        self.conops_lines.append("")

        # Section 5.2 Parameters (Quadratic Physics consistent with v_terminal=1.65, E_k=34.0, m=25.0)
        self.conops_lines.append("### 5.2 Intrinsic Risk Classification & Kinetic Impact Energy Physics Derivations")
        self.conops_lines.append("| Parameter | Symbol | Nominal Value | Units | Constraint / Derivation Rule | Description |")
        self.conops_lines.append("| :--- | :--- | :--- | :--- | :--- | :--- |")
        self.conops_lines.append("| System Operational Mass | m | 25.0 | kg | m <= m_max | Total system mass |")
        self.conops_lines.append("| Gravitational Acceleration | g | 9.80665 | m/s^2 | g = 9.80665 | Standard gravitational constant |")
        self.conops_lines.append("| Atmospheric Air Density | rho | 1.225 | kg/m^3 | rho >= 1.225 | Standard air density |")
        self.conops_lines.append("| Parachute Canopy Area | S_canopy | 84.18 | m^2 | S_canopy >= S_min | Deployed canopy surface area |")
        self.conops_lines.append("| Parachute Drag Coefficient | C_d_parachute | 1.75 | Dimensionless | C_d >= 1.50 | Canopy drag coefficient |")
        self.conops_lines.append("| Parachute Terminal Velocity | v_terminal_parachute | 1.65 | m/s | v <= 1.65 | Descent velocity |")
        self.conops_lines.append("| Mitigated Kinetic Energy | E_k_mitigated | 34.0 | J | E_k <= 34.0 | Mitigated energy |")
        self.conops_lines.append("")

        # 13 MIL-STD-810H methods
        self.conops_lines.append("MIL-STD-810H Methods: Method 500.6, Method 501.7, Method 502.7, Method 503.7, Method 505.7, Method 506.6, Method 507.6, Method 508.8, Method 509.7, Method 510.7, Method 514.8, Method 516.8, Method 521.4.")
        self.conops_lines.append("")
        # 7 EMG rows
        for k in range(1, 8):
            self.conops_lines.append(f"| `EMG-0{k}` | Trigger {k} | Sensor {k} | Action {k} | State {k} | 0.05 s | Role {k} |")
        self.conops_lines.append("")
        self.conops_lines.append("$$")
        self.conops_lines.append(r"P_{\mathrm{EMG-07}} > P_{\mathrm{EMG-06}} > P_{\mathrm{EMG-05}} > P_{\mathrm{EMG-04}} > P_{\mathrm{EMG-03}} > P_{\mathrm{EMG-02}} > P_{\mathrm{EMG-01}}")
        self.conops_lines.append("$$")
        self.conops_lines.append("")
        self.conops_lines.append("EMG-01 Lost C2 loiter and autonomous return to base.")
        self.conops_lines.append("EMG-07 Immediate flight termination and motor cutoff.")

        self.conops_file = os.path.join(self.docs_conops, "CONOPS.md")
        with open(self.conops_file, "w", encoding="utf-8") as f:
            f.write("\n".join(self.conops_lines))

        # Create minimal conforming MISSION_INTENT.md (>=400 lines with 10 sections)
        self.intent_lines = []
        self.intent_lines.append("| Attribute | Value |")
        self.intent_lines.append("| :--- | :--- |")
        self.intent_lines.append("| **Title** | Tactical Mission Intent |")
        self.intent_lines.append("")
        self.intent_lines.append("# Tactical Mission Intent")
        self.intent_lines.append("")
        for i in range(1, 11):
            self.intent_lines.append(f"## {i}. Section {i} Intent Title")
            self.intent_lines.append("")
            for j in range(1, 40):
                self.intent_lines.append(f"Section {i} intent statement line {j} establishing tactical objectives.")
            self.intent_lines.append("")

        # 16 Threat Vectors
        self.intent_lines.append("| Threat ID | Domain | Threat Vector | Technical Description | Severity | Detection Mechanism | Autonomous Mitigation Rule | Public Clause Citation |")
        self.intent_lines.append("| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |")
        for t in range(1, 17):
            self.intent_lines.append(f"| `THR-{t:02d}` | Domain | Threat {t} | Description | High | Detection | Mitigation | Citation |")
        self.intent_lines.append("")
        # 4 PACE Tiers
        self.intent_lines.append("| PACE Tier | Link Medium | Frequency Band (f_band) | Nominal Data Rate (Rate_nom) | Heartbeat Timeout (tau_timeout) | Failover Hysteresis (tau_hysteresis) | Priority / Role | Public Clause Citation |")
        self.intent_lines.append("| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |")
        self.intent_lines.append("| **Primary** | Datalink | f_band | Rate | Timeout | Hysteresis | Role | Citation |")
        self.intent_lines.append("| **Alternate** | LTE Tunnel | f_band | Rate | Timeout | Hysteresis | Role | Citation |")
        self.intent_lines.append("| **Contingency** | Narrowband | f_band | Rate | Timeout | Hysteresis | Role | Citation |")
        self.intent_lines.append("| **Emergency** | Beacon | f_band | Rate | Timeout | Hysteresis | Role | Citation |")
        self.intent_lines.append("")
        # Kalman Table
        self.intent_lines.append("| Parameter | Symbol | Units | Constraint / Rule | Description |")
        self.intent_lines.append("| :--- | :--- | :--- | :--- | :--- |")
        self.intent_lines.append("| A Priori State Covariance | P_k\\|k-1 | m^2 | P_k\\|k-1 > 0 | Predicted covariance |")
        self.intent_lines.append("| Process Noise Covariance | Q_k | m^2/s^2 | Q_k >= 0 | Process noise |")
        self.intent_lines.append("| Kalman Gain Matrix | K_k | Dimensionless | Optimal | Kalman gain |")
        self.intent_lines.append("| Measurement Noise Covariance | R_k | m^2 | R_k > 0 | Measurement noise |")
        self.intent_lines.append("| A Posteriori State Covariance | P_k\\|k | m^2 | norm(P_state) <= norm_P | Updated covariance |")
        self.intent_lines.append("")
        # Bingo Energy Table (2.0 kWh = 7,200,000 J capacity >= 400 W * 4 h * 3600 = 5,760,000 J)
        self.intent_lines.append("| Energy Parameter | Symbol | Value | Units | Constraint Rule | Public Clause Citation |")
        self.intent_lines.append("| :--- | :--- | :--- | :--- | :--- | :--- |")
        self.intent_lines.append("| Total Storage Capacity | E_capacity | 7200000.0 | J | Capacity | Citation |")
        self.intent_lines.append("| Return Transit Energy | E_return | 2500000.0 | J | Return | Citation |")
        self.intent_lines.append("| Secondary Divert Energy | E_divert | 1000000.0 | J | Divert | Citation |")
        self.intent_lines.append("| Mandatory Statutory Reserve | E_reserve | 1500000.0 | J | Reserve | Citation |")
        self.intent_lines.append("| Contingency Buffer | E_contingency | 500000.0 | J | Buffer | Citation |")
        self.intent_lines.append("| Total Bingo Threshold | E_bingo | 5500000.0 | J | Threshold | Citation |")
        self.intent_lines.append("")
        self.intent_lines.append("NIST SP 800-82r3 anti-replay protection with monotonic sequence counters.")

        self.intent_file = os.path.join(self.docs_conops, "MISSION_INTENT.md")
        with open(self.intent_file, "w", encoding="utf-8") as f:
            f.write("\n".join(self.intent_lines))

    def tearDown(self):
        shutil.rmtree(self.temp_dir, ignore_errors=True)

    def test_layer1_delivery_gate_success(self):
        res = verify_layer1_delivery_gate(self.temp_dir)
        self.assertTrue(res.passed, f"Layer 1 failed unexpectedly: {res.errors}")
        self.assertEqual(len(res.errors), 0)

    def test_layer1_delivery_gate_missing_file(self):
        os.remove(self.intent_file)
        res = verify_layer1_delivery_gate(self.temp_dir)
        self.assertFalse(res.passed)
        self.assertTrue(any("MISSION_INTENT.md does not exist" in e for e in res.errors))

    def test_layer1_delivery_gate_undersized_lines(self):
        with open(self.conops_file, "w", encoding="utf-8") as f:
            f.write("# Short Conops\n## 1. Section 1\nLine 1\n")
        res = verify_layer1_delivery_gate(self.temp_dir)
        self.assertFalse(res.passed)
        self.assertTrue(any("total lines" in e for e in res.errors))

    def test_layer2_syntax_purity_success(self):
        res = verify_layer2_syntax_purity(self.temp_dir)
        self.assertTrue(res.passed, f"Layer 2 failed unexpectedly: {res.errors}")

    def test_layer2_mustache_token_failure(self):
        with open(self.conops_file, "a", encoding="utf-8") as f:
            f.write("\nUnrendered token: {{SYSTEM_ID}}\n")
        res = verify_layer2_syntax_purity(self.temp_dir)
        self.assertFalse(res.passed)
        self.assertTrue(any("mustache" in e.lower() for e in res.errors))

    def test_layer2_pseudovariable_failure(self):
        with open(self.intent_file, "a", encoding="utf-8") as f:
            f.write("\nUninstantiated variable: Ao_threshold = 0.99\n")
        res = verify_layer2_syntax_purity(self.temp_dir)
        self.assertFalse(res.passed)
        self.assertTrue(any("pseudovariable" in e.lower() for e in res.errors))

    def test_layer2_raw_dollar_in_table_cell(self):
        with open(self.conops_file, "a", encoding="utf-8") as f:
            f.write("\n| Bad Cell | $x + y$ | Value |\n")
        res = verify_layer2_syntax_purity(self.temp_dir)
        self.assertFalse(res.passed)
        self.assertTrue(any("table cell" in e.lower() for e in res.errors))

    def test_layer3_cardinality_success(self):
        res = verify_layer3_cardinality(self.temp_dir)
        self.assertTrue(res.passed, f"Layer 3 failed unexpectedly: {res.errors}")

    def test_layer3_missing_threat_vector(self):
        with open(self.intent_file, "r", encoding="utf-8") as f:
            content = f.read()
        content = content.replace("THR-16", "INVALID-THR")
        with open(self.intent_file, "w", encoding="utf-8") as f:
            f.write(content)
        res = verify_layer3_cardinality(self.temp_dir)
        self.assertFalse(res.passed)
        self.assertTrue(any("THR-16" in e for e in res.errors))

    def test_layer3_missing_emg_row(self):
        with open(self.conops_file, "r", encoding="utf-8") as f:
            content = f.read()
        content = content.replace("EMG-07", "EMG-XX")
        with open(self.conops_file, "w", encoding="utf-8") as f:
            f.write(content)
        res = verify_layer3_cardinality(self.temp_dir)
        self.assertFalse(res.passed)
        self.assertTrue(any("EMG-07" in e for e in res.errors))

    def test_layer4_physical_math_success(self):
        res = verify_layer4_physical_math(self.temp_dir)
        self.assertTrue(res.passed, f"Layer 4 failed unexpectedly: {res.errors}")

    def test_layer4_kinetic_energy_breach(self):
        with open(self.conops_file, "r", encoding="utf-8") as f:
            content = f.read()
        content = content.replace("34.0 | J | E_k <= 34.0", "50.0 | J | E_k <= 34.0")
        with open(self.conops_file, "w", encoding="utf-8") as f:
            f.write(content)
        res = verify_layer4_physical_math(self.temp_dir)
        self.assertFalse(res.passed)
        self.assertTrue(any("34.0" in e for e in res.errors))

    def test_layer4_bingo_conservation_failure(self):
        with open(self.intent_file, "r", encoding="utf-8") as f:
            content = f.read()
        content = content.replace("5500000.0 | J | Threshold", "6000000.0 | J | Threshold")
        with open(self.intent_file, "w", encoding="utf-8") as f:
            f.write(content)
        res = verify_layer4_physical_math(self.temp_dir)
        self.assertFalse(res.passed)
        self.assertTrue(any("conservation" in e.lower() for e in res.errors))

    def test_layer5_adversarial_invariants_success(self):
        res = verify_layer5_adversarial_invariants(self.temp_dir)
        self.assertTrue(res.passed, f"Layer 5 failed unexpectedly: {res.errors}")

    def test_layer5_missing_priority_arbitration(self):
        with open(self.conops_file, "r", encoding="utf-8") as f:
            content = f.read()
        content = content.replace(r"P_{\mathrm{EMG-07}}", "INVALID_PRIORITY")
        with open(self.conops_file, "w", encoding="utf-8") as f:
            f.write(content)
        res = verify_layer5_adversarial_invariants(self.temp_dir)
        self.assertFalse(res.passed)
        self.assertTrue(any("priority" in e.lower() for e in res.errors))

    def test_layer5_positive_lexicon_floor_failure(self):
        # Purge aviation terms from CONOPS and Intent
        with open(self.conops_file, "r", encoding="utf-8") as f:
            c = f.read()
        for term in ["Airframe", "Avionics", "Payload", "SORA", "airframe", "avionics", "payload", "sora"]:
            c = c.replace(term, "GenericComponent")
        with open(self.conops_file, "w", encoding="utf-8") as f:
            f.write(c)

        with open(self.intent_file, "r", encoding="utf-8") as f:
            ic = f.read()
        for term in ["Airframe", "Avionics", "Payload", "SORA", "airframe", "avionics", "payload", "sora"]:
            ic = ic.replace(term, "GenericComponent")
        with open(self.intent_file, "w", encoding="utf-8") as f:
            f.write(ic)

        res = verify_layer5_adversarial_invariants(self.temp_dir)
        self.assertFalse(res.passed)
        self.assertTrue(any("lexicon floor breach" in e for e in res.errors))

    def test_layer5_pairwise_similarity_plagiarism_failure(self):
        # Create domain_texts dictionary where temp_dir has 100% sentence overlap with a cloned domain
        with open(self.conops_file, "r", encoding="utf-8") as f:
            c = f.read()
        with open(self.intent_file, "r", encoding="utf-8") as f:
            ic = f.read()
        txt = c + "\n" + ic
        domain_id = os.path.basename(os.path.abspath(self.temp_dir))
        domain_texts = {
            domain_id: txt,
            "run_02_plagiarized_clone": txt,
        }
        res = verify_layer5_adversarial_invariants(self.temp_dir, domain_texts=domain_texts)
        self.assertFalse(res.passed)
        self.assertTrue(any("anti-plagiarism" in e or "Pairwise domain similarity" in e for e in res.errors))


# ---------------------------------------------------------------------------
# Unit Tests for the 5 Deterministic Semantic Solvers
# ---------------------------------------------------------------------------

class TestDeterministicSemanticSolvers(unittest.TestCase):
    
    # -----------------------------------------------------------------------
    # Solver 1: Relational Table Mass Cross-Sum Solver
    # -----------------------------------------------------------------------
    def test_solver1_relational_mass_cross_sum_success(self):
        table_md = """
### 1.3.2 Parametric Subsystem Mass/Resource Budget Breakdown Table
| Structural Group (AST Partition) | Allocated Subsystems & Components | Mass Fraction (% MTOW) | Mass Budget (kg) | Nominal Power Budget (W) | Peak Power Budget (W) |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Airframe Structure** | Fuselage primary structure | 34.0% | 8.50 | 0.0 | 0.0 |
| **Avionics & Processing** | Flight control computer | 12.8% | 3.20 | 45.0 | 75.0 |
| **Propulsion & Power Distribution** | Actuators and electric motors | 24.0% | 6.00 | 300.0 | 500.0 |
| **Energy Storage Subsystem** | Smart battery module | 18.0% | 4.50 | 0.0 | 0.0 |
| **Primary Mission Payload** | Multi-modal mission sensor suite | 8.0% | 2.00 | 45.0 | 80.0 |
| **Autonomous Failsafe Containment** | Independent safety watchdog | 3.2% | 0.80 | 10.0 | 25.0 |
| **Total System Integration** | **Integrated Platform** | **100.0% MTOW** | **25.0** | **400.0** | **680.0** |
"""
        passed, errors, details = solve_relational_mass_cross_sum(table_md)
        self.assertTrue(passed, f"Solver 1 failed unexpectedly: {errors}")
        self.assertEqual(len(errors), 0)
        self.assertAlmostEqual(details["sum_partition_masses"], 25.0, places=4)
        self.assertAlmostEqual(details["total_mtow_kg"], 25.0, places=4)

    def test_solver1_relational_mass_cross_sum_mismatch_failure(self):
        # Change payload mass to 3.0 kg, making sum 26.0 kg != 25.0 kg
        table_md = """
### 1.3.2 Parametric Subsystem Mass/Resource Budget Breakdown Table
| Structural Group (AST Partition) | Allocated Subsystems & Components | Mass Fraction (% MTOW) | Mass Budget (kg) | Nominal Power Budget (W) | Peak Power Budget (W) |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Airframe Structure** | Fuselage primary structure | 34.0% | 8.50 | 0.0 | 0.0 |
| **Avionics & Processing** | Flight control computer | 12.8% | 3.20 | 45.0 | 75.0 |
| **Propulsion & Power Distribution** | Actuators and electric motors | 24.0% | 6.00 | 300.0 | 500.0 |
| **Energy Storage Subsystem** | Smart battery module | 18.0% | 4.50 | 0.0 | 0.0 |
| **Primary Mission Payload** | Multi-modal mission sensor suite | 12.0% | 3.00 | 45.0 | 80.0 |
| **Autonomous Failsafe Containment** | Independent safety watchdog | 3.2% | 0.80 | 10.0 | 25.0 |
| **Total System Integration** | **Integrated Platform** | **100.0% MTOW** | **25.0** | **400.0** | **680.0** |
"""
        passed, errors, details = solve_relational_mass_cross_sum(table_md)
        self.assertFalse(passed)
        self.assertTrue(any("Mass Cross-Sum mismatch" in e for e in errors))

    def test_solver1_relational_mass_missing_partition_failure(self):
        # Omit Failsafe partition
        table_md = """
### 1.3.2 Parametric Subsystem Mass/Resource Budget Breakdown Table
| Structural Group (AST Partition) | Allocated Subsystems & Components | Mass Fraction (% MTOW) | Mass Budget (kg) | Nominal Power Budget (W) | Peak Power Budget (W) |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Airframe Structure** | Fuselage primary structure | 34.0% | 8.50 | 0.0 | 0.0 |
| **Avionics & Processing** | Flight control computer | 12.8% | 3.20 | 45.0 | 75.0 |
| **Propulsion & Power Distribution** | Actuators and electric motors | 24.0% | 6.00 | 300.0 | 500.0 |
| **Energy Storage Subsystem** | Smart battery module | 18.0% | 4.50 | 0.0 | 0.0 |
| **Primary Mission Payload** | Multi-modal mission sensor suite | 8.0% | 2.00 | 45.0 | 80.0 |
| **Total System Integration** | **Integrated Platform** | **100.0% MTOW** | **25.0** | **400.0** | **680.0** |
"""
        passed, errors, details = solve_relational_mass_cross_sum(table_md)
        self.assertFalse(passed)
        self.assertTrue(any("missing AST partition" in e for e in errors))

    # -----------------------------------------------------------------------
    # Solver 2: Closed-Form Quadratic Physics Solver
    # -----------------------------------------------------------------------
    def test_solver2_closed_form_quadratic_physics_success(self):
        # m=25.0, g=9.80665, rho=1.225, S=84.18, C_d=1.75
        # v_calc = sqrt(2*25*9.80665 / (1.225*84.18*1.75)) = 1.6483 m/s (~1.65 m/s)
        # E_k_calc = 0.5 * 25 * 1.6483^2 = 33.96 J (~34.0 J)
        section52_md = """
### 5.2 Intrinsic Risk Classification & Kinetic Impact Energy Physics Derivations
| Parameter | Symbol | Nominal Value | Units | Constraint / Derivation Rule | Description |
| :--- | :--- | :--- | :--- | :--- | :--- |
| System Operational Mass | m | 25.0 | kg | m <= m_max | Total system mass |
| Gravitational Acceleration | g | 9.80665 | m/s^2 | g = 9.80665 | Standard gravitational constant |
| Atmospheric Air Density | rho | 1.225 | kg/m^3 | rho >= 1.225 | ISA air density |
| Parachute Canopy Area | S_canopy | 84.18 | m^2 | S_canopy >= S_min | Deployed recovery canopy area |
| Parachute Drag Coefficient | C_d_parachute | 1.75 | Dimensionless | C_d >= 1.50 | Canopy drag coefficient |
| Parachute Terminal Velocity | v_terminal_parachute | 1.65 | m/s | v <= 1.65 | Descent velocity |
| Mitigated Kinetic Energy | E_k_mitigated | 34.0 | J | E_k <= 34.0 | Mitigated energy |
"""
        passed, errors, details = solve_closed_form_quadratic_physics(section52_md)
        self.assertTrue(passed, f"Solver 2 failed unexpectedly: {errors}")
        self.assertAlmostEqual(details["v_calc_mps"], 1.6483, places=2)
        self.assertAlmostEqual(details["E_k_calc_J"], 33.96, places=1)

    def test_solver2_closed_form_quadratic_physics_velocity_mismatch_failure(self):
        # Tabulated terminal velocity declared as 3.5 m/s while calculated is ~1.65 m/s
        section52_md = """
### 5.2 Intrinsic Risk Classification & Kinetic Impact Energy Physics Derivations
| Parameter | Symbol | Nominal Value | Units | Constraint / Derivation Rule | Description |
| :--- | :--- | :--- | :--- | :--- | :--- |
| System Operational Mass | m | 25.0 | kg | m <= m_max | Total system mass |
| Gravitational Acceleration | g | 9.80665 | m/s^2 | g = 9.80665 | Standard gravitational constant |
| Atmospheric Air Density | rho | 1.225 | kg/m^3 | rho >= 1.225 | ISA air density |
| Parachute Canopy Area | S_canopy | 84.18 | m^2 | S_canopy >= S_min | Deployed recovery canopy area |
| Parachute Drag Coefficient | C_d_parachute | 1.75 | Dimensionless | C_d >= 1.50 | Canopy drag coefficient |
| Parachute Terminal Velocity | v_terminal_parachute | 3.50 | m/s | v <= 3.50 | Descent velocity |
| Mitigated Kinetic Energy | E_k_mitigated | 34.0 | J | E_k <= 34.0 | Mitigated energy |
"""
        passed, errors, details = solve_closed_form_quadratic_physics(section52_md)
        self.assertFalse(passed)
        self.assertTrue(any("terminal velocity v_terminal" in e for e in errors))

    def test_solver2_closed_form_quadratic_physics_energy_mismatch_failure(self):
        # Tabulated mitigated energy declared as 90.0 J while calculated is ~34.0 J
        section52_md = """
### 5.2 Intrinsic Risk Classification & Kinetic Impact Energy Physics Derivations
| Parameter | Symbol | Nominal Value | Units | Constraint / Derivation Rule | Description |
| :--- | :--- | :--- | :--- | :--- | :--- |
| System Operational Mass | m | 25.0 | kg | m <= m_max | Total system mass |
| Gravitational Acceleration | g | 9.80665 | m/s^2 | g = 9.80665 | Standard gravitational constant |
| Atmospheric Air Density | rho | 1.225 | kg/m^3 | rho >= 1.225 | ISA air density |
| Parachute Canopy Area | S_canopy | 84.18 | m^2 | S_canopy >= S_min | Deployed recovery canopy area |
| Parachute Drag Coefficient | C_d_parachute | 1.75 | Dimensionless | C_d >= 1.50 | Canopy drag coefficient |
| Parachute Terminal Velocity | v_terminal_parachute | 1.65 | m/s | v <= 1.65 | Descent velocity |
| Mitigated Kinetic Energy | E_k_mitigated | 90.0 | J | E_k <= 90.0 | Mitigated energy |
"""
        passed, errors, details = solve_closed_form_quadratic_physics(section52_md)
        self.assertFalse(passed)
        self.assertTrue(any("mitigated kinetic energy E_k_mitigated" in e for e in errors))

    def test_solver2_closed_form_quadratic_physics_missing_param_failure(self):
        # Omit canopy area
        section52_md = """
### 5.2 Intrinsic Risk Classification & Kinetic Impact Energy Physics Derivations
| Parameter | Symbol | Nominal Value | Units | Constraint / Derivation Rule | Description |
| :--- | :--- | :--- | :--- | :--- | :--- |
| System Operational Mass | m | 25.0 | kg | m <= m_max | Total system mass |
| Parachute Terminal Velocity | v_terminal_parachute | 1.65 | m/s | v <= 1.65 | Descent velocity |
| Mitigated Kinetic Energy | E_k_mitigated | 34.0 | J | E_k <= 34.0 | Mitigated energy |
"""
        passed, errors, details = solve_closed_form_quadratic_physics(section52_md)
        self.assertFalse(passed)
        self.assertTrue(any("missing required parameters" in e for e in errors))

    # -----------------------------------------------------------------------
    # Solver 3: Dimensional Scaling & Energy Conservation Engine
    # -----------------------------------------------------------------------
    def test_solver3_dimensional_energy_conservation_kwh_success(self):
        conops_md = """
| Structural Group (AST Partition) | Allocated Subsystems & Components | Mass Fraction (% MTOW) | Mass Budget (kg) | Nominal Power Budget (W) | Peak Power Budget (W) |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Total System Integration** | **Integrated Platform** | **100.0% MTOW** | **25.0** | **400.0** | **680.0** |

| Parameter ID | Bounding Parameter Name | Parametric Symbol | Threshold (Boundary Limit) | Objective (Nominal Target) | Engineering Unit | Normative / Safety Basis |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **PL-09** | Mission Operational Endurance | t_endurance | >= 4.0 | 4.0 | hours | Mission endurance |
"""
        intent_md = """
| Energy Parameter | Symbol | Value | Units | Constraint Rule | Public Clause Citation |
| :--- | :--- | :--- | :--- | :--- | :--- |
| Total Storage Capacity | E_capacity | 2.5 | kWh | Capacity | Citation |
"""
        # Capacity = 2.5 kWh = 9,000,000 J >= 400 W * (4.0 * 3600) = 5,760,000 J
        passed, errors, details = solve_dimensional_energy_conservation(conops_md, intent_md)
        self.assertTrue(passed, f"Solver 3 failed unexpectedly: {errors}")
        self.assertAlmostEqual(details["e_capacity_joules"], 9000000.0)
        self.assertAlmostEqual(details["e_required_joules"], 5760000.0)

    def test_solver3_dimensional_energy_conservation_joules_success(self):
        conops_md = """
| Parameter ID | Bounding Parameter Name | Parametric Symbol | Threshold (Boundary Limit) | Objective (Nominal Target) | Engineering Unit | Normative / Safety Basis |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **PL-09** | Mission Operational Endurance | t_endurance | >= 180.0 | 180.0 | min | Mission endurance |
"""
        intent_md = """
| Energy Parameter | Symbol | Value | Units | Constraint Rule | Public Clause Citation |
| :--- | :--- | :--- | :--- | :--- | :--- |
| Total Storage Capacity | E_capacity | 6000000.0 | J | Capacity | Citation |
| Nominal Power | P_nominal | 500.0 | W | Nominal Power | Citation |
"""
        # Required = 500 W * (180/60 * 3600) = 500 * 10800 = 5,400,000 J <= 6,000,000 J
        passed, errors, details = solve_dimensional_energy_conservation(conops_md, intent_md)
        self.assertTrue(passed, f"Solver 3 failed unexpectedly: {errors}")
        self.assertAlmostEqual(details["e_required_joules"], 5400000.0)

    def test_solver3_dimensional_energy_conservation_insufficient_capacity_failure(self):
        conops_md = """
| Parameter ID | Bounding Parameter Name | Parametric Symbol | Threshold (Boundary Limit) | Objective (Nominal Target) | Engineering Unit | Normative / Safety Basis |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **PL-09** | Mission Operational Endurance | t_endurance | >= 4.0 | 4.0 | hours | Mission endurance |
"""
        intent_md = """
| Energy Parameter | Symbol | Value | Units | Constraint Rule | Public Clause Citation |
| :--- | :--- | :--- | :--- | :--- | :--- |
| Total Storage Capacity | E_capacity | 1.0 | kWh | Capacity | Citation |
| Nominal Power | P_nominal | 500.0 | W | Nominal Power | Citation |
"""
        # Declared = 1.0 kWh = 3.6e6 J < Required = 500 W * 4 h * 3600 = 7.2e6 J
        passed, errors, details = solve_dimensional_energy_conservation(conops_md, intent_md)
        self.assertFalse(passed)
        self.assertTrue(any("Dimensional Energy Conservation Violation" in e for e in errors))

    # -----------------------------------------------------------------------
    # Solver 4: Normative Standards Cross-Checker
    # -----------------------------------------------------------------------
    def test_solver4_normative_standards_domain_config_success(self):
        conops_md = """
### 1.5 Normative Standards & Regulatory Baseline
| Standard ID | Issuing Body | Title |
| :--- | :--- | :--- |
| RTCA DO-178C | RTCA | Airborne Software |
| ISO 26262 | ISO | Functional Safety |
| IEC 62304 | IEC | Medical Device Software |
"""
        config = {
            "REGULATORY_STANDARDS": ["RTCA DO-178C", "IEC 62304"]
        }
        passed, errors, details = solve_normative_standards_cross_check(
            workspace_path="/dummy", conops_text=conops_md, domain_config=config
        )
        self.assertTrue(passed, f"Solver 4 failed: {errors}")

    def test_solver4_normative_standards_missing_standard_failure(self):
        conops_md = """
### 1.5 Normative Standards & Regulatory Baseline
| Standard ID | Issuing Body | Title |
| :--- | :--- | :--- |
| RTCA DO-178C | RTCA | Airborne Software |
"""
        config = {
            "REGULATORY_STANDARDS": ["IEC 62304"]
        }
        passed, errors, details = solve_normative_standards_cross_check(
            workspace_path="/dummy", conops_text=conops_md, domain_config=config
        )
        self.assertFalse(passed)
        self.assertTrue(any("IEC 62304" in e for e in errors))

    def test_solver4_normative_standards_medical_domain(self):
        # Positive
        conops_pass = "### 1.5 Normative Standards\nGoverned by IEC 62304 for medical device software."
        passed, _, _ = solve_normative_standards_cross_check(
            workspace_path="/test_projects/run_07_medical_robot",
            conops_text=conops_pass,
            domain_config={"domain": "Medical"}
        )
        self.assertTrue(passed)

        # Negative
        conops_fail = "### 1.5 Normative Standards\nGoverned by generic software standards."
        passed, errors, _ = solve_normative_standards_cross_check(
            workspace_path="/test_projects/run_07_medical_robot",
            conops_text=conops_fail,
            domain_config={"domain": "Medical"}
        )
        self.assertFalse(passed)
        self.assertTrue(any("IEC 62304" in e for e in errors))

    def test_solver4_normative_standards_rail_domain(self):
        conops_pass = "### 1.5 Normative Standards\nGoverned by EN 50128 for railway safety."
        passed, _, _ = solve_normative_standards_cross_check(
            workspace_path="/test_projects/run_08_rail_locomotive",
            conops_text=conops_pass,
            domain_config={"domain": "Autonomous Rail"}
        )
        self.assertTrue(passed)

        conops_fail = "### 1.5 Normative Standards\nGoverned by DO-178C."
        passed, errors, _ = solve_normative_standards_cross_check(
            workspace_path="/test_projects/run_08_rail_locomotive",
            conops_text=conops_fail,
            domain_config={"domain": "Autonomous Rail"}
        )
        self.assertFalse(passed)
        self.assertTrue(any("EN 50128" in e for e in errors))

    def test_solver4_normative_standards_space_domain(self):
        conops_pass = "### 1.5 Normative Standards\nGoverned by ECSS-E-ST-40C space engineering standard."
        passed, _, _ = solve_normative_standards_cross_check(
            workspace_path="/test_projects/run_06_leo_cubesat",
            conops_text=conops_pass,
            domain_config={"domain": "LEO CubeSat Space"}
        )
        self.assertTrue(passed)

        conops_fail = "### 1.5 Normative Standards\nGoverned by DO-178C."
        passed, errors, _ = solve_normative_standards_cross_check(
            workspace_path="/test_projects/run_06_leo_cubesat",
            conops_text=conops_fail,
            domain_config={"domain": "Space"}
        )
        self.assertFalse(passed)
        self.assertTrue(any("ECSS" in e for e in errors))

    def test_solver4_normative_standards_agv_domain(self):
        conops_pass = "### 1.5 Normative Standards\nGoverned by ISO 3691-4 industrial AGV safety standard."
        passed, _, _ = solve_normative_standards_cross_check(
            workspace_path="/test_projects/run_09_industrial_agv",
            conops_text=conops_pass,
            domain_config={"domain": "Industrial Forklift AGV"}
        )
        self.assertTrue(passed)

        conops_fail = "### 1.5 Normative Standards\nGoverned by DO-178C."
        passed, errors, _ = solve_normative_standards_cross_check(
            workspace_path="/test_projects/run_09_industrial_agv",
            conops_text=conops_fail,
            domain_config={"domain": "Industrial Forklift AGV"}
        )
        self.assertFalse(passed)
        self.assertTrue(any("ISO 3691-4" in e for e in errors))

    def test_solver4_normative_standards_subsea_domain(self):
        conops_pass = "### 1.5 Normative Standards\nGoverned by DNV-GL marine rules for underwater AUVs."
        passed, _, _ = solve_normative_standards_cross_check(
            workspace_path="/test_projects/run_03_subsea_auv",
            conops_text=conops_pass,
            domain_config={"domain": "Subsea AUV"}
        )
        self.assertTrue(passed)

        conops_fail = "### 1.5 Normative Standards\nGoverned by DO-178C."
        passed, errors, _ = solve_normative_standards_cross_check(
            workspace_path="/test_projects/run_03_subsea_auv",
            conops_text=conops_fail,
            domain_config={"domain": "Subsea AUV"}
        )
        self.assertFalse(passed)
        self.assertTrue(any("DNV-GL" in e for e in errors))

    # -----------------------------------------------------------------------
    # Solver 5: Forbidden Cross-Domain Ontology Scanner
    # -----------------------------------------------------------------------
    def test_solver5_forbidden_cross_domain_tactical_isr_clean(self):
        # Military Aircraft domain: airframe, parachute, AGL, ROE, PID are allowed
        conops = "Tactical ISR UAV with frangible airframe, parachute recovery, altitude AGL, and ROE-01."
        intent = "PID verification and weapons release protocols."
        passed, errors, _ = solve_forbidden_cross_domain_ontology(
            workspace_path="/test_projects/run_01_tactical_isr",
            conops_text=conops,
            intent_text=intent,
            domain_config={"is_aircraft": True, "is_civilian": False}
        )
        self.assertTrue(passed, f"Solver 5 failed: {errors}")

    def test_solver5_forbidden_cross_domain_non_aircraft_airframe_failure(self):
        conops = "Subsea autonomous underwater vehicle chassis and airframe structure."
        passed, errors, _ = solve_forbidden_cross_domain_ontology(
            workspace_path="/test_projects/run_03_subsea_auv",
            conops_text=conops,
            domain_config={"is_aircraft": False}
        )
        self.assertFalse(passed)
        self.assertTrue(any("airframe" in e for e in errors))

    def test_solver5_forbidden_cross_domain_non_aircraft_parachute_failure(self):
        conops = "Autonomous ground vehicle with emergency recovery parachute."
        passed, errors, _ = solve_forbidden_cross_domain_ontology(
            workspace_path="/test_projects/run_05_ground_ugv",
            conops_text=conops,
            domain_config={"is_aircraft": False}
        )
        self.assertFalse(passed)
        self.assertTrue(any("parachute" in e for e in errors))

    def test_solver5_forbidden_cross_domain_non_aircraft_agl_failure(self):
        conops = "LEO CubeSat constellation operating at altitude AGL."
        passed, errors, _ = solve_forbidden_cross_domain_ontology(
            workspace_path="/test_projects/run_06_leo_cubesat",
            conops_text=conops,
            domain_config={"is_aircraft": False}
        )
        self.assertFalse(passed)
        self.assertTrue(any("altitude AGL" in e for e in errors))

    def test_solver5_forbidden_cross_domain_non_aircraft_v_stall_failure(self):
        conops = "| Parameter | Symbol | Nominal | Units |\n| Stall Speed | V_stall | <= 15.0 | m/s |"
        passed, errors, _ = solve_forbidden_cross_domain_ontology(
            workspace_path="/test_projects/run_08_rail_locomotive",
            conops_text=conops,
            domain_config={"is_aircraft": False}
        )
        self.assertFalse(passed)
        self.assertTrue(any("V_stall" in e for e in errors))

    def test_solver5_forbidden_cross_domain_non_aircraft_remote_id_failure(self):
        conops = "Robotic surgical console complying with ASTM F3411 Remote ID."
        passed, errors, _ = solve_forbidden_cross_domain_ontology(
            workspace_path="/test_projects/run_07_medical_robot",
            conops_text=conops,
            domain_config={"is_aircraft": False}
        )
        self.assertFalse(passed)
        self.assertTrue(any("Remote ID" in e for e in errors))

    def test_solver5_forbidden_cross_domain_civilian_roe_failure(self):
        conops = "Medical robotic console governed by ROE-01 and ROE-02."
        passed, errors, _ = solve_forbidden_cross_domain_ontology(
            workspace_path="/test_projects/run_07_medical_robot",
            conops_text=conops,
            domain_config={"is_civilian": True}
        )
        self.assertFalse(passed)
        self.assertTrue(any("ROE" in e for e in errors))

    def test_solver5_forbidden_cross_domain_civilian_pid_failure(self):
        conops = "Industrial forklift AGV executing PID target positive identification."
        passed, errors, _ = solve_forbidden_cross_domain_ontology(
            workspace_path="/test_projects/run_09_industrial_agv",
            conops_text=conops,
            domain_config={"is_civilian": True}
        )
        self.assertFalse(passed)
        self.assertTrue(any("PID" in e for e in errors))

    def test_solver5_forbidden_cross_domain_civilian_weapons_release_failure(self):
        conops = "Autonomous rail locomotive with automated weapons release interlocks."
        passed, errors, _ = solve_forbidden_cross_domain_ontology(
            workspace_path="/test_projects/run_08_rail_locomotive",
            conops_text=conops,
            domain_config={"is_civilian": True}
        )
        self.assertFalse(passed)
        self.assertTrue(any("weapons release" in e for e in errors))

    def test_solver5_forbidden_cross_domain_civilian_collateral_damage_failure(self):
        conops = "Autonomous ground delivery fleet minimizing collateral damage radius."
        passed, errors, _ = solve_forbidden_cross_domain_ontology(
            workspace_path="/test_projects/run_05_ground_ugv",
            conops_text=conops,
            domain_config={"is_civilian": True}
        )
        self.assertFalse(passed)
        self.assertTrue(any("collateral damage" in e for e in errors))

    # -----------------------------------------------------------------------
    # Solver 6: Positive Domain Lexicon Density Floor
    # -----------------------------------------------------------------------
    def test_solver6_positive_lexicon_medical_success(self):
        text = "The surgeon utilizes a trocar, laparoscope, master manipulator, sterile drape, and end-effector with haptic feedback, DICOM and HL7."
        passed, errors, details = solve_positive_domain_lexicon_floor("run_07", "medical", text)
        self.assertTrue(passed, f"Solver 6 medical failed: {errors}")
        self.assertGreaterEqual(details["matched_count"], 4)
        self.assertIn("surgeon", details["matched_terms"])

    def test_solver6_positive_lexicon_medical_failure(self):
        text = "The surgeon reviewed the DICOM image on the terminal."
        passed, errors, details = solve_positive_domain_lexicon_floor("run_07", "medical", text)
        self.assertFalse(passed)
        self.assertEqual(details["matched_count"], 2)
        self.assertTrue(any("lexicon floor breach" in e for e in errors))
        self.assertIn("trocar", details["missing_terms"])

    def test_solver6_positive_lexicon_rail_success(self):
        text = "The track circuit communicates with the axle counter and coupler in the shunting yard. ETCS balises monitor speed."
        passed, errors, details = solve_positive_domain_lexicon_floor("run_08", "rail", text)
        self.assertTrue(passed, f"Solver 6 rail failed: {errors}")
        self.assertGreaterEqual(details["matched_count"], 4)

    def test_solver6_positive_lexicon_rail_failure(self):
        text = "The rail train stopped at the station coupler."
        passed, errors, details = solve_positive_domain_lexicon_floor("run_08", "rail", text)
        self.assertFalse(passed)
        self.assertLess(details["matched_count"], 4)

    def test_solver6_positive_lexicon_marine_success(self):
        text = "Bathymetry survey utilizing buoyancy engine, USBL, DVL, CTD, and transponder acoustic modem under COLREGs."
        passed, errors, details = solve_positive_domain_lexicon_floor("run_03", "marine", text)
        self.assertTrue(passed, f"Solver 6 marine failed: {errors}")
        self.assertGreaterEqual(details["matched_count"], 4)

    def test_solver6_positive_lexicon_marine_failure(self):
        text = "The underwater vessel navigated the seaway."
        passed, errors, details = solve_positive_domain_lexicon_floor("run_03", "marine", text)
        self.assertFalse(passed)
        self.assertLess(details["matched_count"], 4)

    def test_solver6_positive_lexicon_space_success(self):
        text = "ADCS reaction wheels and magnetorquers maintain pointing using star tracker in LVLH frame during orbital eclipse with CCSDS telemetry tracking."
        passed, errors, details = solve_positive_domain_lexicon_floor("run_06", "space", text)
        self.assertTrue(passed, f"Solver 6 space failed: {errors}")
        self.assertGreaterEqual(details["matched_count"], 4)

    def test_solver6_positive_lexicon_space_failure(self):
        text = "The satellite payload transmitted telemetry."
        passed, errors, details = solve_positive_domain_lexicon_floor("run_06", "space", text)
        self.assertFalse(passed)
        self.assertLess(details["matched_count"], 4)

    def test_solver6_positive_lexicon_industrial_success(self):
        text = "Pallet handling via fork mast during docking per VDA 5050 with safety field protection, optical lidar, and odometry."
        passed, errors, details = solve_positive_domain_lexicon_floor("run_09", "industrial", text)
        self.assertTrue(passed, f"Solver 6 industrial failed: {errors}")
        self.assertGreaterEqual(details["matched_count"], 4)

    def test_solver6_positive_lexicon_industrial_failure(self):
        text = "The warehouse vehicle executed docking maneuvers."
        passed, errors, details = solve_positive_domain_lexicon_floor("run_09", "industrial", text)
        self.assertFalse(passed)
        self.assertLess(details["matched_count"], 4)

    def test_solver6_positive_lexicon_aviation_success(self):
        text = "The airframe and flight controller navigate regulated airspace with aerodynamic wings, avionics, SORA, and payload."
        passed, errors, details = solve_positive_domain_lexicon_floor("run_01", "aviation", text)
        self.assertTrue(passed, f"Solver 6 aviation failed: {errors}")
        self.assertGreaterEqual(details["matched_count"], 4)

    def test_solver6_positive_lexicon_aviation_failure(self):
        text = "The system carried a sensor payload."
        passed, errors, details = solve_positive_domain_lexicon_floor("run_01", "aviation", text)
        self.assertFalse(passed)
        self.assertLess(details["matched_count"], 4)

    def test_infer_lexicon_domain_type_airspace_collision_avoidance(self):
        """Verify _infer_lexicon_domain_type does not misclassify airspace as space (Fixes Issue #186)."""
        # Airspace operations should infer aviation, not space
        inferred = _infer_lexicon_domain_type("uas-airspace-monitoring", "Airspace Operations", "CONOPS for Airspace Surveillance")
        self.assertEqual(inferred, "aviation")

        # Deep space orbital cubesat should infer space
        inferred_space = _infer_lexicon_domain_type("leo-cubesat", "Deep Space Orbital CubeSat", "CONOPS for Satellite Constellation")
        self.assertEqual(inferred_space, "space")

    # -----------------------------------------------------------------------
    # Solver 7: Pairwise Anti-Plagiarism Gate
    # -----------------------------------------------------------------------
    def test_solver7_extract_substantive_sentences(self):
        md_text = """
# Header Line
| Header 1 | Header 2 |
| :--- | :--- |
| **Short** | Hi |
| **Long Cell** | This is a substantive table cell sentence describing the system. |

This is the first substantive prose sentence. Here is the second detailed sentence with technical parameters.
Short.
- Item bullet point containing four words here.
"""
        sentences = extract_substantive_sentences(md_text)
        self.assertIn("this is a substantive table cell sentence describing the system", sentences)
        self.assertIn("this is the first substantive prose sentence", sentences)
        self.assertIn("here is the second detailed sentence with technical parameters", sentences)
        self.assertIn("item bullet point containing four words here", sentences)
        self.assertNotIn("short", sentences)
        self.assertNotIn("hi", sentences)

    def test_solver7_pairwise_similarity_clean_success(self):
        domain_texts = {
            "medical_robot": "The surgical console operates with sterile drapes and trocars in laparoscopic theaters. Master manipulators provide haptic feedback.",
            "rail_locomotive": "The autonomous rail shunting locomotive connects brake pipes and monitors axle counters. Bogie dynamics are governed by ETCS balises.",
        }
        passed, errors, details = solve_pairwise_domain_similarity(domain_texts, max_threshold=0.25)
        self.assertTrue(passed, f"Solver 7 failed: {errors}")
        self.assertEqual(len(errors), 0)
        self.assertLessEqual(details["max_similarity"], 0.25)

    def test_solver7_pairwise_similarity_plagiarism_failure(self):
        shared_text = (
            "The system operates with autonomous failover architecture.\n"
            "All telemetry streams are cryptographically validated every cycle.\n"
            "Emergency braking activates immediately upon watchdog signal loss.\n"
            "Redundant power supplies maintain nominal operation during voltage dips.\n"
            "Environmental testing conforms to standard procedures and thermal baselines."
        )
        domain_texts = {
            "domain_alpha": shared_text + "\nDomain alpha specific distinct sentence here.",
            "domain_beta": shared_text + "\nDomain beta specific distinct sentence here.",
        }
        passed, errors, details = solve_pairwise_domain_similarity(domain_texts, max_threshold=0.25)
        self.assertFalse(passed)
        self.assertGreater(details["max_similarity"], 0.25)
        self.assertTrue(any("anti-plagiarism threshold" in e for e in errors))

    def test_solver7_pairwise_similarity_custom_threshold(self):
        shared_text = "Sentence one about system telemetry.\nSentence two about power supply distribution.\nSentence three about environmental validation."
        domain_texts = {
            "domain_a": shared_text + "\nSpecific sentence for domain A here.",
            "domain_b": shared_text + "\nSpecific sentence for domain B here.",
        }
        passed, errors, details = solve_pairwise_domain_similarity(domain_texts, max_threshold=0.90)
        self.assertTrue(passed)


class TestScorecardAndReportGeneration(unittest.TestCase):
    def test_report_generation(self):
        domain = DomainScorecard(
            domain_id="run_01",
            domain_name="Tactical ISR Fixed-Wing UAV",
            workspace_path="/tmp/test_projects/run_01_tactical_isr",
            overall_passed=True,
            layers={
                1: LayerResult(layer_id=1, layer_name="Delivery Gate 0", passed=True),
                2: LayerResult(layer_id=2, layer_name="Mechanical Syntax & Token Purity", passed=True),
                3: LayerResult(layer_id=3, layer_name="Statutory Cardinality", passed=True),
                4: LayerResult(layer_id=4, layer_name="Closed-Form Physical & Math Solver", passed=True),
                5: LayerResult(layer_id=5, layer_name="Adversarial Invariant Verification", passed=True),
                6: LayerResult(layer_id=6, layer_name="Baseline Parity & Model Coverage", passed=True),
            }
        )
        summary = HarnessSummary(
            total_domains=1,
            passed_domains=1,
            failed_domains=0,
            execution_timestamp="2026-09-03T22:00:00Z",
            domain_results=[domain],
            similarity_matrix={"run_01": {"run_01": 1.0}}
        )
        md_text = generate_markdown_report(summary)
        self.assertIn("# Master 10-Domain E2E Acceptance Test Report", md_text)
        self.assertIn("Tactical ISR Fixed-Wing UAV", md_text)
        self.assertIn("## Pairwise Anti-Plagiarism & Cross-Domain Similarity Matrix", md_text)
        self.assertIn("PASS", md_text)

        json_text = generate_json_scorecard(summary)
        data = json.loads(json_text)
        self.assertEqual(data["total_domains"], 1)
        self.assertEqual(data["passed_domains"], 1)
        self.assertIn("similarity_matrix", data)


if __name__ == "__main__":
    unittest.main()

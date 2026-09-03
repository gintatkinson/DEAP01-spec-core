#!/usr/bin/env python3
"""
Test suite for the End-to-End Acceptance Harness (scripts/e2e_acceptance_harness.py).
Validates each of the 6 verification layers and report generation.
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

# Import harness components
from scripts.e2e_acceptance_harness import (
    AcceptanceHarness,
    LayerResult,
    DomainScorecard,
    HarnessSummary,
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
        self.conops_lines.append("| **Title** | Concept of Operations: Test Archetype |")
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
        
        # Add SORA params and EMG matrix
        self.conops_lines.append("| Parameter | Symbol | Nominal Value | Units | Constraint / Derivation Rule | Description |")
        self.conops_lines.append("| :--- | :--- | :--- | :--- | :--- | :--- |")
        self.conops_lines.append("| System Operational Mass | m | 25.0 | kg | m <= m_max | Total system mass |")
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
        # Bingo Energy Table
        self.intent_lines.append("| Energy Parameter | Symbol | Value | Units | Constraint Rule | Public Clause Citation |")
        self.intent_lines.append("| :--- | :--- | :--- | :--- | :--- | :--- |")
        self.intent_lines.append("| Total Storage Capacity | E_capacity | 500000.0 | J | Capacity | Citation |")
        self.intent_lines.append("| Return Transit Energy | E_return | 150000.0 | J | Return | Citation |")
        self.intent_lines.append("| Secondary Divert Energy | E_divert | 60000.0 | J | Divert | Citation |")
        self.intent_lines.append("| Mandatory Statutory Reserve | E_reserve | 100000.0 | J | Reserve | Citation |")
        self.intent_lines.append("| Contingency Buffer | E_contingency | 40000.0 | J | Buffer | Citation |")
        self.intent_lines.append("| Total Bingo Threshold | E_bingo | 350000.0 | J | Threshold | Citation |")
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
        # Corrupt threat matrix by removing THR-16
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
        # Change E_bingo from 350000 to 400000 without changing parts
        content = content.replace("350000.0 | J | Threshold", "400000.0 | J | Threshold")
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


class TestScorecardAndReportGeneration(unittest.TestCase):
    def test_report_generation(self):
        domain = DomainScorecard(
            domain_id="run_01",
            domain_name="Tactical ISR Fixed-Wing UAV",
            workspace_path="/Users/perkunas/test_projects/run_01_tactical_isr",
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
            domain_results=[domain]
        )
        md_text = generate_markdown_report(summary)
        self.assertIn("# Master 10-Domain E2E Acceptance Test Report", md_text)
        self.assertIn("Tactical ISR Fixed-Wing UAV", md_text)
        self.assertIn("PASS", md_text)

        json_text = generate_json_scorecard(summary)
        data = json.loads(json_text)
        self.assertEqual(data["total_domains"], 1)
        self.assertEqual(data["passed_domains"], 1)


if __name__ == "__main__":
    unittest.main()

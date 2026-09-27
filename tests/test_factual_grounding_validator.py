"""
Unit tests for FactualGroundingValidator: Positive AST Provenance & Anti-Regex Hardening.

Covers remediation of:
- Issue #378: Negative-string regex heuristic for [TIER-3] creates gate evasion
- Issue #376: Regex only matches bracketed [TIER-3] syntax, allowing evasion via (Declared Assumption)
- Issue #364: Code block fence unconditionally bypasses numeric grounding in Mermaid diagrams
"""

import os
import sys
import tempfile
import unittest
import warnings

repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if repo_root not in sys.path:
    sys.path.insert(0, repo_root)

parity_src = os.path.join(repo_root, "skills", "spec-orchestrator", "parity_auditor", "src")
if parity_src not in sys.path:
    sys.path.insert(0, parity_src)

from parity_auditor.core.workspace import WorkspaceRepository
from parity_auditor.validators.factual_grounding_validator import (
    FactualGroundingValidator,
    _has_epistemic_exemption,
    EPISTEMIC_EXEMPTION_PATTERN,
)


SAMPLE_SYSML_AST = """package AutonomousVehicle_SSOT {
    doc /* Authoritative Ground Truth for Flight Vehicle */

    attribute ruddervatorCount : Integer = 4;
    attribute tailConfiguration : String = "X-tail";
    attribute catapultLaunchLimitG : Real = 12.0;
    attribute maxGLoad : Real = 12.0;
    attribute massKg : Real = 25.0;
    attribute nominalVoltageV : Real = 24.0;
    attribute telemetryRateHz : Real = 50.0;

    part def FlightControlComputer {
        port c2_bus : RS485;
        port telemetry : MAVLink;
        attribute clockFrequencyHz : Real = 400.0;
    }

    part def BatterySystem {
        attribute maxVoltageV : Real = 28.0;
    }

    part def Propulsion {
        attribute propulsionPeakPowerW : Real = 2800.0;
        part def DriveMotor {
            attribute motorPeakPowerW : Real = 3000.0;
        }
    }
}
"""


class TestFactualGroundingValidatorAntiRegex(unittest.TestCase):
    """Tests for Issues #378 and #376: Deprecation and elimination of epistemic exemptions."""

    def setUp(self):
        self.validator = FactualGroundingValidator()

    def test_epistemic_exemption_helper_deprecated_and_returns_false(self):
        """_has_epistemic_exemption must be deprecated and return False to prevent gate evasion."""
        test_strings = [
            "The ESC uses DShot600 [TIER-3: DESIGN]",
            "The attitude controller executes at 400 Hz (TIER-3: DESIGN)",
            "Timeout is 500 ms [TIER-4: TBD]",
            "Actuator bandwidth is 50 Hz (Declared Assumption)",
            "Declared Assumption: motor current limit is 40A",
            "[TIER-3] Some claim",
            "(TIER-3) Some claim",
        ]
        for s in test_strings:
            with warnings.catch_warnings(record=True) as w:
                warnings.simplefilter("always")
                res = _has_epistemic_exemption(s)
                self.assertFalse(res, f"Expected False for '{s}', got {res}")
                self.assertTrue(any(issubclass(item.category, DeprecationWarning) for item in w))

    def test_tier3_tag_cannot_bypass_numeric_drift(self):
        """Ungrounded numeric claims with [TIER-3: DESIGN] fail closed."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_SYSML_AST)

            # Airframe mass is declared as 25.0 kg. Claiming 45.0 kg exceeds limit.
            doc_content = """# Propulsion and Structure
The airframe total mass is 45.0 kg [TIER-3: DESIGN].
The battery system maximum voltage is 48.0 V [TIER-3: DESIGN].
The drive motor peak power is 3500 W [TIER-3: DESIGN].
"""
            with open(os.path.join(docs_dir, "feat-propulsion.md"), "w", encoding="utf-8") as f:
                f.write(doc_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])

            rule_ids = {f.rule_id for f in findings}
            self.assertIn("factual-grounding-numeric-drift", rule_ids)
            findings_text = " ".join(f.message for f in findings)
            self.assertIn("45.0 kg", findings_text)
            self.assertIn("3500", findings_text)

    def test_declared_assumption_cannot_bypass_protocol_or_rate_grounding(self):
        """Ungrounded protocol or execution rate with (Declared Assumption) fails closed."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_SYSML_AST)

            # STANAG 4586 is not declared in SAMPLE_SYSML_AST
            doc_content = """# Datalink Architecture
The mission payload communicates over STANAG 4586 datalink (Declared Assumption).
The inner loop rate executes at 1000 Hz inner loop (Declared Assumption).
"""
            with open(os.path.join(docs_dir, "feat-datalink.md"), "w", encoding="utf-8") as f:
                f.write(doc_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])

            rule_ids = {f.rule_id for f in findings}
            self.assertIn("factual-grounding-unverified-protocol", rule_ids)
            findings_text = " ".join(f.message for f in findings)
            self.assertIn("STANAG 4586", findings_text)

    def test_tier4_tag_cannot_bypass_structural_count(self):
        """Ungrounded structural count assertion with [TIER-4: TBD] fails closed."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_SYSML_AST)

            # Schema declares 4 ruddervators. Claiming 2 ruddervators must fail.
            doc_content = """# Empennage
The empennage features 2 ruddervators [TIER-4: TBD] for pitch and yaw control.
"""
            with open(os.path.join(docs_dir, "feat-empennage.md"), "w", encoding="utf-8") as f:
                f.write(doc_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])

            rule_ids = {f.rule_id for f in findings}
            self.assertIn("factual-grounding-numeric-drift", rule_ids)
            findings_text = " ".join(f.message for f in findings)
            self.assertIn("ruddervators", findings_text)

    def test_positive_grounded_claims_pass(self):
        """Claims adhering to AST ground truth pass cleanly with zero findings."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_SYSML_AST)

            doc_content = """# Grounded Subsystem
The vehicle has 4 ruddervators arranged in an X-tail configuration.
Catapult launch acceleration is limited to 12.0g.
Total airframe mass is 25.0 kg.
Drive motor peak power is 3000 W.
"""
            with open(os.path.join(docs_dir, "feat-grounded.md"), "w", encoding="utf-8") as f:
                f.write(doc_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])
            self.assertEqual(findings, [])


class TestFactualGroundingValidatorMermaidHardening(unittest.TestCase):
    """Tests for Issue #364: Numbers in Mermaid diagrams and code blocks cannot bypass grounding."""

    def setUp(self):
        self.validator = FactualGroundingValidator()

    def test_mermaid_sequence_diagram_ungrounded_numeric_fails(self):
        """Ungrounded frequency in Mermaid sequence diagram fails closed."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_SYSML_AST)

            doc_content = """# Flight Control Telemetry
```mermaid
sequenceDiagram
    participant FCC as FlightControlComputer
    participant ESC as ElectronicSpeedController
    FCC ->> ESC: Motor throttle command at 800 Hz
```
"""
            with open(os.path.join(docs_dir, "feat-seq.md"), "w", encoding="utf-8") as f:
                f.write(doc_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])

            rule_ids = {f.rule_id for f in findings}
            self.assertIn("factual-grounding-numeric-drift", rule_ids)
            findings_text = " ".join(f.message for f in findings)
            self.assertIn("800 Hz", findings_text)

    def test_mermaid_sequence_diagram_grounded_numeric_passes(self):
        """Grounded frequency in Mermaid sequence diagram passes."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_SYSML_AST)

            doc_content = """# Flight Control Telemetry
```mermaid
sequenceDiagram
    participant FCC as FlightControlComputer
    participant Ground as GroundStation
    FCC ->> Ground: Periodic telemetry stream at 50 Hz
```
"""
            with open(os.path.join(docs_dir, "feat-seq-grounded.md"), "w", encoding="utf-8") as f:
                f.write(doc_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])
            self.assertEqual(findings, [])

    def test_mermaid_state_diagram_exceeded_limit_fails(self):
        """State diagram transition asserting an ungrounded or exceeded bound fails closed."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_SYSML_AST)

            # Catapult launch acceleration is 12.0g; claiming 25.0g fails
            doc_content = """# Launch State Machine
```mermaid
stateDiagram-v2
    [*] --> Standby
    Standby --> RailLaunch : LaunchCommand [accel <= 25g]
    RailLaunch --> InFlight : Separation
```
"""
            with open(os.path.join(docs_dir, "feat-state.md"), "w", encoding="utf-8") as f:
                f.write(doc_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])

            rule_ids = {f.rule_id for f in findings}
            self.assertIn("factual-grounding-numeric-drift", rule_ids)
            findings_text = " ".join(f.message for f in findings)
            self.assertIn("25g", findings_text)

    def test_mermaid_state_diagram_grounded_limit_passes(self):
        """State diagram transition with grounded limit passes."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_SYSML_AST)

            doc_content = """# Launch State Machine
```mermaid
stateDiagram-v2
    [*] --> Standby
    Standby --> RailLaunch : LaunchCommand [accel <= 12g]
    RailLaunch --> InFlight : Separation
```
"""
            with open(os.path.join(docs_dir, "feat-state-pass.md"), "w", encoding="utf-8") as f:
                f.write(doc_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])
            self.assertEqual(findings, [])

    def test_mermaid_flowchart_ungrounded_numeric_fails(self):
        """Mermaid flowchart with ungrounded numeric edge or node fails closed."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_SYSML_AST)

            doc_content = """# Sampling Flowchart
```mermaid
flowchart TD
    Sensor[IMU Sensor] -->|Sample at 1000 Hz| Filter[EKF Filter]
```
"""
            with open(os.path.join(docs_dir, "feat-flowchart.md"), "w", encoding="utf-8") as f:
                f.write(doc_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])

            rule_ids = {f.rule_id for f in findings}
            self.assertIn("factual-grounding-numeric-drift", rule_ids)
            findings_text = " ".join(f.message for f in findings)
            self.assertIn("1000 Hz", findings_text)

    def test_mermaid_class_diagram_ungrounded_property_fails(self):
        """Mermaid class diagram with ungrounded electrical or physical quantity fails."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_SYSML_AST)

            # Battery maximum voltage is 28.0 V; 96.0 V is ungrounded / exceeded
            doc_content = """# Battery System Class
```mermaid
classDiagram
    class BatterySystem {
        +maxVoltage: 96.0 V
    }
```
"""
            with open(os.path.join(docs_dir, "feat-class.md"), "w", encoding="utf-8") as f:
                f.write(doc_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])

            rule_ids = {f.rule_id for f in findings}
            self.assertIn("factual-grounding-numeric-drift", rule_ids)

    def test_markdown_code_block_ungrounded_numeric_fails(self):
        """Markdown code block (non-mermaid) with ungrounded physical quantity fails closed."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_SYSML_AST)

            doc_content = """# Configuration Payload
```yaml
avionics:
  sample_frequency: 950 Hz
```
"""
            with open(os.path.join(docs_dir, "feat-yaml.md"), "w", encoding="utf-8") as f:
                f.write(doc_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])

            rule_ids = {f.rule_id for f in findings}
            self.assertIn("factual-grounding-numeric-drift", rule_ids)

    def test_markdown_code_block_with_ssot_citation_passes(self):
        """Markdown code block with comment-level SSOT citation passes."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_SYSML_AST)

            doc_content = """# Grounded Configuration
```yaml
# Source: schema/model.sysml §FlightControlComputer
clock_frequency: 400.0 Hz
```
"""
            with open(os.path.join(docs_dir, "feat-yaml-cited.md"), "w", encoding="utf-8") as f:
                f.write(doc_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])
            self.assertEqual(findings, [])


if __name__ == "__main__":
    unittest.main()

"""
Unit and integration tests for Check 25: Cross-Document Diagram Parity Gate.

Verifies:
1. Symbol export in parity_auditor.validators and registration in AGGREGATING_VALIDATORS.
2. Passes when Mermaid architecture diagrams match 1:1 between CONOPS.md and executive deliverables.
3. Detects and fails on missing nodes.
4. Detects and fails on extra undeclared nodes.
5. Detects and fails on missing embedded port attributes.
6. Detects and fails on modified port attributes (direction or data type).
7. Detects and fails on missing connection links (edges).
8. Detects and fails on altered connection labels or connection IDs.
9. Detects and fails on subgraph mismatches.
10. Gracefully passes on clean landing zones (no specs / .gitkeep only).
11. Integration with scripts/verify_downstream_baseline.py Check 25 gate.
"""

import os
import sys
import tempfile
import unittest
from pathlib import Path

repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if repo_root not in sys.path:
    sys.path.insert(0, repo_root)

parity_src = os.path.join(repo_root, "skills", "spec-orchestrator", "parity_auditor", "src")
if parity_src not in sys.path:
    sys.path.insert(0, parity_src)

from parity_auditor.core.workspace import WorkspaceRepository
from parity_auditor.validators.cross_document_diagram_parity_validator import (
    CrossDocumentDiagramParityValidator,
    validate_cross_document_diagram_parity,
    RULE_ID,
)
from parity_auditor.validators.architecture_viewpoint_validator import (
    ArchitectureViewpointValidator,
    RULE_OV6C_MISSING_SCENARIO_DIAGRAM,
    _validate_ov6c_scenario_coverage,
)
from parity_auditor.aggregator import AGGREGATING_VALIDATORS
from scripts.verify_downstream_baseline import (
    check_cross_document_diagram_parity,
    check_cross_document_diagram_parity_gate,
)

SAMPLE_CONOPS_SV1 = """# Concept of Operations (ConOps)

## 4.8 Physical Subsystem Architecture (DoDAF SV-1)

```mermaid
flowchart TD
    subgraph GroundSegment ["Ground Control & RF Link Segment"]
        direction TB
        GCS["SwarmC2GroundStation<br/>(GroundControlStation)<br/>• c2Uplink (OUT: C2Commands)<br/>• c2Downlink (IN: C2Telemetry)<br/>• videoIn (IN: VideoStream)"]
        GSR["GroundStationRadio<br/>• rfl (INOUT: RFLink)"]
    end

    subgraph PrimaryVehicleSegment ["Primary Vehicle Segment (AlphaPlatform)"]
        direction TB
        OBC["OnboardComputer<br/>• c2Uplink (IN: C2Commands)<br/>• c2Downlink (OUT: C2Telemetry)<br/>• actuatorCmdOut (OUT: ActuatorCommand)<br/>• dscIn1Out (OUT: Discrete_ArmEnable)"]
        ESAD["ESAD<br/>• dscIn1 (IN: Discrete_ArmEnable)<br/>• initTrain (OUT: Initiation_Train)"]
        WH["WarheadModule<br/>• initTrain (IN: Initiation_Train)"]
        ACT["Actuation<br/>• actuatorCmdIn (IN: ActuatorCommand)"]
    end

    GCS -->|"CONN-01: C2Commands (C2 Uplink)"| OBC
    OBC -->|"CONN-02: C2Telemetry (C2 Downlink)"| GCS
    OBC -->|"CONN-03: Discrete_ArmEnable (1 kHz PWM)"| ESAD
    ESAD -->|"CONN-04: Initiation_Train (Detonation Energy)"| WH
    OBC -->|"CONN-05: ActuatorCommand (Flight Demand)"| ACT
```
"""

SAMPLE_DELIVERABLES_MATCHING = """# Executive Deliverables: Phase 1 Synthesis

## Deliverable 3: System Functional Architecture

### 3.1 System Functional & Logical Architecture (DoDAF SV-1 / IEEE 1362 §5.3)

```mermaid
flowchart TD
    subgraph GroundSegment ["Ground Control & RF Link Segment"]
        direction TB
        GCS["SwarmC2GroundStation<br/>(GroundControlStation)<br/>• c2Uplink (OUT: C2Commands)<br/>• c2Downlink (IN: C2Telemetry)<br/>• videoIn (IN: VideoStream)"]
        GSR["GroundStationRadio<br/>• rfl (INOUT: RFLink)"]
    end

    subgraph PrimaryVehicleSegment ["Primary Vehicle Segment (AlphaPlatform)"]
        direction TB
        OBC["OnboardComputer<br/>• c2Uplink (IN: C2Commands)<br/>• c2Downlink (OUT: C2Telemetry)<br/>• actuatorCmdOut (OUT: ActuatorCommand)<br/>• dscIn1Out (OUT: Discrete_ArmEnable)"]
        ESAD["ESAD<br/>• dscIn1 (IN: Discrete_ArmEnable)<br/>• initTrain (OUT: Initiation_Train)"]
        WH["WarheadModule<br/>• initTrain (IN: Initiation_Train)"]
        ACT["Actuation<br/>• actuatorCmdIn (IN: ActuatorCommand)"]
    end

    GCS -->|"CONN-01: C2Commands (C2 Uplink)"| OBC
    OBC -->|"CONN-02: C2Telemetry (C2 Downlink)"| GCS
    OBC -->|"CONN-03: Discrete_ArmEnable (1 kHz PWM)"| ESAD
    ESAD -->|"CONN-04: Initiation_Train (Detonation Energy)"| WH
    OBC -->|"CONN-05: ActuatorCommand (Flight Demand)"| ACT
```
"""


class TestCrossDocumentDiagramParity(unittest.TestCase):
    def setUp(self):
        self.validator = CrossDocumentDiagramParityValidator()

    def test_registration_in_aggregating_validators_and_init(self):
        """Verify validator is registered in AGGREGATING_VALIDATORS."""
        self.assertIn(CrossDocumentDiagramParityValidator, AGGREGATING_VALIDATORS)
        from parity_auditor.validators import (
            CrossDocumentDiagramParityValidator as ExportedValidator,
            validate_cross_document_diagram_parity as exported_fn,
        )
        self.assertEqual(ExportedValidator, CrossDocumentDiagramParityValidator)
        self.assertTrue(callable(exported_fn))

    def test_cross_document_diagram_parity_success_when_matching(self):
        """Verify 0 findings when CONOPS and report architecture diagrams match 1:1."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            reports_dir = os.path.join(tmpdir, "docs", "reports")
            os.makedirs(conops_dir, exist_ok=True)
            os.makedirs(reports_dir, exist_ok=True)

            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_CONOPS_SV1)

            with open(os.path.join(reports_dir, "PHASE_1_EXECUTIVE_ENGINEERING_DELIVERABLES.md"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_DELIVERABLES_MATCHING)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo)
            self.assertEqual(findings, [])

            fn_errors = validate_cross_document_diagram_parity(tmpdir)
            self.assertEqual(fn_errors, [])

    def test_detects_missing_node(self):
        """Verify failure when a node (e.g. WH) is omitted in executive deliverables."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            reports_dir = os.path.join(tmpdir, "docs", "reports")
            os.makedirs(conops_dir, exist_ok=True)
            os.makedirs(reports_dir, exist_ok=True)

            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_CONOPS_SV1)

            # Report missing WH node
            report_missing_node = SAMPLE_DELIVERABLES_MATCHING.replace(
                'WH["WarheadModule<br/>• initTrain (IN: Initiation_Train)"]\n', ''
            ).replace(
                'ESAD -->|"CONN-04: Initiation_Train (Detonation Energy)"| WH\n', ''
            )

            with open(os.path.join(reports_dir, "PHASE_1_EXECUTIVE_ENGINEERING_DELIVERABLES.md"), "w", encoding="utf-8") as f:
                f.write(report_missing_node)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo)
            self.assertTrue(len(findings) >= 1)
            self.assertTrue(any("Missing node 'WH'" in str(f) for f in findings))

    def test_detects_extra_undeclared_node(self):
        """Verify failure when an extra phantom node is present in executive deliverables."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            reports_dir = os.path.join(tmpdir, "docs", "reports")
            os.makedirs(conops_dir, exist_ok=True)
            os.makedirs(reports_dir, exist_ok=True)

            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_CONOPS_SV1)

            # Report contains extra node UNKNOWN_PAYLOAD
            report_extra = SAMPLE_DELIVERABLES_MATCHING.replace(
                'WH["WarheadModule<br/>• initTrain (IN: Initiation_Train)"]',
                'WH["WarheadModule<br/>• initTrain (IN: Initiation_Train)"]\n        EXTRA["ExtraPhantomNode"]'
            )

            with open(os.path.join(reports_dir, "PHASE_1_EXECUTIVE_ENGINEERING_DELIVERABLES.md"), "w", encoding="utf-8") as f:
                f.write(report_extra)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo)
            self.assertTrue(len(findings) >= 1)
            self.assertTrue(any("Extra undeclared node 'EXTRA'" in str(f) for f in findings))

    def test_detects_missing_port_attribute(self):
        """Verify failure when a declared port bullet is missing on a node."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            reports_dir = os.path.join(tmpdir, "docs", "reports")
            os.makedirs(conops_dir, exist_ok=True)
            os.makedirs(reports_dir, exist_ok=True)

            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_CONOPS_SV1)

            # Remove dscIn1Out port from OBC
            report_missing_port = SAMPLE_DELIVERABLES_MATCHING.replace(
                '<br/>• dscIn1Out (OUT: Discrete_ArmEnable)', ''
            )

            with open(os.path.join(reports_dir, "PHASE_1_EXECUTIVE_ENGINEERING_DELIVERABLES.md"), "w", encoding="utf-8") as f:
                f.write(report_missing_port)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo)
            self.assertTrue(len(findings) >= 1)
            self.assertTrue(any("missing port 'dscIn1Out'" in str(f) for f in findings))

    def test_detects_modified_port_attribute(self):
        """Verify failure when port direction or data type is altered in executive report."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            reports_dir = os.path.join(tmpdir, "docs", "reports")
            os.makedirs(conops_dir, exist_ok=True)
            os.makedirs(reports_dir, exist_ok=True)

            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_CONOPS_SV1)

            # Alter dscIn1Out direction from OUT to IN
            report_modified_port = SAMPLE_DELIVERABLES_MATCHING.replace(
                'dscIn1Out (OUT: Discrete_ArmEnable)',
                'dscIn1Out (IN: Discrete_ArmEnable)'
            )

            with open(os.path.join(reports_dir, "PHASE_1_EXECUTIVE_ENGINEERING_DELIVERABLES.md"), "w", encoding="utf-8") as f:
                f.write(report_modified_port)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo)
            self.assertTrue(len(findings) >= 1)
            self.assertTrue(any("direction modified" in str(f) for f in findings))

    def test_detects_missing_connection_edge(self):
        """Verify failure when an edge link is missing in executive report."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            reports_dir = os.path.join(tmpdir, "docs", "reports")
            os.makedirs(conops_dir, exist_ok=True)
            os.makedirs(reports_dir, exist_ok=True)

            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_CONOPS_SV1)

            # Remove CONN-03 edge
            report_missing_edge = SAMPLE_DELIVERABLES_MATCHING.replace(
                'OBC -->|"CONN-03: Discrete_ArmEnable (1 kHz PWM)"| ESAD\n', ''
            )

            with open(os.path.join(reports_dir, "PHASE_1_EXECUTIVE_ENGINEERING_DELIVERABLES.md"), "w", encoding="utf-8") as f:
                f.write(report_missing_edge)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo)
            self.assertTrue(len(findings) >= 1)
            self.assertTrue(any("Missing connection link 'OBC' -> 'ESAD'" in str(f) for f in findings))

    def test_detects_altered_connection_label(self):
        """Verify failure when connection label / ID is altered."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            reports_dir = os.path.join(tmpdir, "docs", "reports")
            os.makedirs(conops_dir, exist_ok=True)
            os.makedirs(reports_dir, exist_ok=True)

            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_CONOPS_SV1)

            # Alter CONN-03 to CONN-99
            report_altered_label = SAMPLE_DELIVERABLES_MATCHING.replace(
                'CONN-03: Discrete_ArmEnable (1 kHz PWM)',
                'CONN-99: Discrete_ArmEnable (1 kHz PWM)'
            )

            with open(os.path.join(reports_dir, "PHASE_1_EXECUTIVE_ENGINEERING_DELIVERABLES.md"), "w", encoding="utf-8") as f:
                f.write(report_altered_label)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo)
            self.assertTrue(len(findings) >= 1)
            self.assertTrue(any("ID mismatch" in str(f) or "label mismatch" in str(f) for f in findings))

    def test_detects_subgraph_mismatch(self):
        """Verify failure when a subgraph is missing or renamed."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            reports_dir = os.path.join(tmpdir, "docs", "reports")
            os.makedirs(conops_dir, exist_ok=True)
            os.makedirs(reports_dir, exist_ok=True)

            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_CONOPS_SV1)

            # Rename PrimaryVehicleSegment to AirborneSubsystem
            report_mismatched_sg = SAMPLE_DELIVERABLES_MATCHING.replace(
                'subgraph PrimaryVehicleSegment',
                'subgraph AirborneSubsystem'
            )

            with open(os.path.join(reports_dir, "PHASE_1_EXECUTIVE_ENGINEERING_DELIVERABLES.md"), "w", encoding="utf-8") as f:
                f.write(report_mismatched_sg)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo)
            self.assertTrue(len(findings) >= 1)
            self.assertTrue(any("Missing subgraph 'PrimaryVehicleSegment'" in str(f) for f in findings))

    def test_upstream_clean_landing_zone(self):
        """Verify graceful 0 findings on clean landing zones (empty schema/ or .gitkeep)."""
        with tempfile.TemporaryDirectory() as tmpdir:
            os.makedirs(os.path.join(tmpdir, "docs", "conops"), exist_ok=True)
            os.makedirs(os.path.join(tmpdir, "docs", "reports"), exist_ok=True)
            with open(os.path.join(tmpdir, "docs", "conops", ".gitkeep"), "w") as f:
                f.write("")
            with open(os.path.join(tmpdir, "docs", "reports", ".gitkeep"), "w") as f:
                f.write("")

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo)
            self.assertEqual(findings, [])

    def test_baseline_verification_integration(self):
        """Verify check_cross_document_diagram_parity and alias in verify_downstream_baseline.py."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            reports_dir = os.path.join(tmpdir, "docs", "reports")
            os.makedirs(conops_dir, exist_ok=True)
            os.makedirs(reports_dir, exist_ok=True)

            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_CONOPS_SV1)

            with open(os.path.join(reports_dir, "PHASE_1_EXECUTIVE_ENGINEERING_DELIVERABLES.md"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_DELIVERABLES_MATCHING)

            # Should complete without error
            check_cross_document_diagram_parity(tmpdir)
            check_cross_document_diagram_parity_gate(tmpdir)


class TestArchitectureViewpointValidator(unittest.TestCase):
    """Test suite verifying Diagram 7 (OV-6c) scenario-to-diagram parity."""

    def setUp(self):
        self.validator = ArchitectureViewpointValidator()

    def test_ov6c_scenario_coverage_passes_when_all_scenarios_have_diagrams(self):
        """Verify passing when 100% of scenarios have accompanying sequence diagrams."""
        scenario_text = """# Operational Scenarios

### 9.1 Scenario SCN-01: Nominal Sortie Thread
Nominal operational mission lifecycle.

```mermaid
sequenceDiagram
    autonumber
    actor Operator as "Human Operator"
    participant Controller as "System Controller"
    Operator ->> Controller: Initialize_Mission
    Controller -->> Operator: Mission_Ready
```

### 9.2 Scenario SCN-02: GNSS-Denied Navigation
Degraded navigation thread under satellite denial.

```mermaid
%% OV-6c Scenario Lifeline Sequence
sequenceDiagram
    autonumber
    participant Sensors as "Sensor Fusion"
    participant Controller as "System Controller"
    Sensors ->> Controller: Visual_Odometry_Lock
    Controller -->> Sensors: Acknowledge_State
```

### Scenario SCN-03: Lost C2 Link
Communications failover thread.

```mermaid
sequenceDiagram
    autonumber
    participant Router as "PACE Router"
    participant Watchdog as "Safety Watchdog"
    Router ->> Watchdog: Link_Timeout_Breached
    Watchdog -->> Router: Latch_Safe_Mode
```
"""
        findings = _validate_ov6c_scenario_coverage(scenario_text, "docs/conops/CONOPS.md")
        self.assertEqual(len(findings), 0)

    def test_ov6c_scenario_coverage_fails_when_scenario_lacks_diagram(self):
        """Verify finding emitted when a scenario lacks a sequenceDiagram."""
        scenario_text = """# Operational Scenarios

### 9.1 Scenario SCN-01: Nominal Sortie
```mermaid
sequenceDiagram
    participant A as "Actor A"
    participant B as "Actor B"
    A ->> B: Step_1
```

### 9.2 Scenario SCN-02: Tactical Abort
This scenario has only prose and a table, but lacks a sequenceDiagram.

| Step | Action |
| :--- | :--- |
| 1 | Abort execution |

### 9.3 Scenario SCN-03: Bingo Energy Divert
```mermaid
sequenceDiagram
    participant C as "Controller"
    participant P as "Power System"
    C ->> P: Query_Energy
```
"""
        findings = _validate_ov6c_scenario_coverage(
            scenario_text, "docs/conops/units/conops/09_SCENARIOS_AND_TIMELINES.md"
        )
        self.assertEqual(len(findings), 1)
        finding = findings[0]
        self.assertEqual(finding.rule_id, RULE_OV6C_MISSING_SCENARIO_DIAGRAM)
        self.assertIn("SCN-02", str(finding))
        self.assertIn("Diagram 7 (OV-6c)", str(finding))
        self.assertIn("violates 100% scenario lifeline coverage", str(finding))
        self.assertEqual(finding.detail["scenario_id"], "SCN-02")
        self.assertEqual(finding.detail["rule"], "conops-ov6c-missing-scenario-diagram")

    def test_ov6c_subsections_do_not_prematurely_close_scenario(self):
        """Verify level 4 subsections do not cut off scenario inspection before the diagram."""
        scenario_text = """# Operational Scenarios

### 9.5 Scenario SCN-05: Controlled Safety Interlock Action
Scenario SCN-05 details.

#### 9.5.1 Safety Interlock Prerequisites
Prerequisites text.

#### 9.5.2 Lifeline Sequence
```mermaid
sequenceDiagram
    actor Operator as "Human Operator"
    participant Controller as "System Controller"
    Operator ->> Controller: Consent_Token
```

### 9.6 Scenario SCN-06: SORA Containment Termination
Next scenario.

```mermaid
sequenceDiagram
    participant Watchdog as "Safety Watchdog"
    participant Actuator as "Flight Termination"
    Watchdog ->> Actuator: Trigger_Termination
```
"""
        findings = _validate_ov6c_scenario_coverage(scenario_text, "docs/conops/CONOPS.md")
        self.assertEqual(len(findings), 0)

    def test_validate_markdown_content_wires_ov6c_scenario_check(self):
        """Verify validate_markdown_content executes scenario coverage check."""
        scenario_text = """# CONOPS

### 9.1 Scenario SCN-01: Nominal Sortie
Text without any diagram.
"""
        findings = self.validator.validate_markdown_content(scenario_text, "docs/conops/CONOPS.md")
        missing_scenario_findings = [
            f for f in findings if f.rule_id == RULE_OV6C_MISSING_SCENARIO_DIAGRAM
        ]
        self.assertEqual(len(missing_scenario_findings), 1)
        self.assertIn("SCN-01", str(missing_scenario_findings[0]))

    def test_validate_markdown_content_respects_target_diagrams_filter(self):
        """Verify validate_markdown_content skips scenario check when target_diagrams excludes OV-6c."""
        scenario_text = """# CONOPS

### 9.1 Scenario SCN-01: Nominal Sortie
Text without any diagram.
"""
        findings = self.validator.validate_markdown_content(
            scenario_text, "docs/conops/CONOPS.md", target_diagrams={"SV-1"}
        )
        missing_scenario_findings = [
            f for f in findings if f.rule_id == RULE_OV6C_MISSING_SCENARIO_DIAGRAM
        ]
        self.assertEqual(len(missing_scenario_findings), 0)

    def test_validate_wires_ov6c_scenario_check_in_workspace(self):
        """Verify validate() flags missing scenario diagrams in CONOPS.md."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(conops_dir, exist_ok=True)
            conops_file = os.path.join(conops_dir, "CONOPS.md")

            with open(conops_file, "w", encoding="utf-8") as f:
                f.write("""# Concept of Operations

### 9.1 Scenario SCN-01: Nominal Lifecycle
Only prose, no sequence diagram.
""")

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, allow_missing_specs=True)
            missing_scenario_findings = [
                f for f in findings if f.rule_id == RULE_OV6C_MISSING_SCENARIO_DIAGRAM
            ]
            self.assertEqual(len(missing_scenario_findings), 1)
            self.assertIn("SCN-01", str(missing_scenario_findings[0]))


if __name__ == "__main__":
    unittest.main()

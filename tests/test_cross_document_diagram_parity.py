"""
Unit and integration tests for Check 25: Cross-Document Diagram Parity Gate (Issue #349).

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
12. ArchitectureViewpointValidator canonical viewpoints (OV-6c, SV-4, OV-5a, OV-4, SPATIAL-4D).
13. Consumption of canonical units from skills/spec-conops-engineering/resources/.
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
    RULE_INCOMPLETE_DIAGRAM,
    RULE_DIAGRAM_MISSING,
    _validate_ov6c_scenario_coverage,
    _matches_canonical_diagram,
    _extract_blocks_and_syntax_findings,
    CANONICAL_DIAGRAMS,
    CONOPS_CANONICAL_DIAGRAMS,
)
from parity_auditor.aggregator import AGGREGATING_VALIDATORS
from scripts.verify_downstream_baseline import (
    check_cross_document_diagram_parity,
    check_cross_document_diagram_parity_gate,
)

CANONICAL_UNITS_DIR = os.path.join(
    repo_root, "skills", "spec-conops-engineering", "resources", "units", "conops"
)

# Canonical abstract MBSE architecture diagram (zero hardcoded domain concepts)
SAMPLE_CONOPS_SV1 = """# Concept of Operations (ConOps)

## 4.8 Physical Subsystem Architecture (DoDAF SV-1)

```mermaid
flowchart TD
    subgraph GroundSegment ["Ground Control & Telemetry Segment"]
        direction TB
        GCS["GroundControlStation<br/>• cmdUplink (OUT: CommandVector)<br/>• telemDownlink (IN: TelemetryStream)<br/>• auxData (IN: PayloadStream)"]
        GSR["GroundStationRadio<br/>• rfl (INOUT: RFLink)"]
    end

    subgraph OperationalSegment ["Primary Operational Segment (SystemCore)"]
        direction TB
        CC["CoreController<br/>• cmdUplink (IN: CommandVector)<br/>• telemDownlink (OUT: TelemetryStream)<br/>• actuatorCmdOut (OUT: ActuatorCommand)<br/>• watchdogPulseOut (OUT: HeartbeatPulse)"]
        WD["SafetyWatchdog<br/>• heartbeatIn (IN: HeartbeatPulse)<br/>• failsafeCmd (OUT: FailsafeCommand)"]
        SEN["SensorSuite<br/>• sensorDataOut (OUT: SensorStream)"]
        ACT["ActuatorSubsystem<br/>• actuatorCmdIn (IN: ActuatorCommand)"]
    end

    GCS -->|"CONN-01: CommandVector (Uplink Command)"| CC
    CC -->|"CONN-02: TelemetryStream (Downlink Telemetry)"| GCS
    CC -->|"CONN-03: HeartbeatPulse (10 Hz Pulse)"| WD
    WD -->|"CONN-04: FailsafeCommand (Emergency Containment)"| ACT
    CC -->|"CONN-05: ActuatorCommand (Control Demand)"| ACT
```
"""

SAMPLE_DELIVERABLES_MATCHING = """# Executive Deliverables: Phase 1 Synthesis

## Deliverable 3: System Functional Architecture

### 3.1 System Functional & Logical Architecture (DoDAF SV-1 / IEEE 1362 §5.3)

```mermaid
flowchart TD
    subgraph GroundSegment ["Ground Control & Telemetry Segment"]
        direction TB
        GCS["GroundControlStation<br/>• cmdUplink (OUT: CommandVector)<br/>• telemDownlink (IN: TelemetryStream)<br/>• auxData (IN: PayloadStream)"]
        GSR["GroundStationRadio<br/>• rfl (INOUT: RFLink)"]
    end

    subgraph OperationalSegment ["Primary Operational Segment (SystemCore)"]
        direction TB
        CC["CoreController<br/>• cmdUplink (IN: CommandVector)<br/>• telemDownlink (OUT: TelemetryStream)<br/>• actuatorCmdOut (OUT: ActuatorCommand)<br/>• watchdogPulseOut (OUT: HeartbeatPulse)"]
        WD["SafetyWatchdog<br/>• heartbeatIn (IN: HeartbeatPulse)<br/>• failsafeCmd (OUT: FailsafeCommand)"]
        SEN["SensorSuite<br/>• sensorDataOut (OUT: SensorStream)"]
        ACT["ActuatorSubsystem<br/>• actuatorCmdIn (IN: ActuatorCommand)"]
    end

    GCS -->|"CONN-01: CommandVector (Uplink Command)"| CC
    CC -->|"CONN-02: TelemetryStream (Downlink Telemetry)"| GCS
    CC -->|"CONN-03: HeartbeatPulse (10 Hz Pulse)"| WD
    WD -->|"CONN-04: FailsafeCommand (Emergency Containment)"| ACT
    CC -->|"CONN-05: ActuatorCommand (Control Demand)"| ACT
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

    def test_consume_canonical_conops_units_resource(self):
        """Verify reading genuine canonical architecture from skills/spec-conops-engineering/resources/."""
        arch_file = os.path.join(CANONICAL_UNITS_DIR, "04_SYSTEM_ARCHITECTURE.md")
        self.assertTrue(os.path.isfile(arch_file), f"Canonical unit missing: {arch_file}")
        with open(arch_file, "r", encoding="utf-8") as f:
            content = f.read()

        self.assertIn("CoreController", content)
        self.assertIn("SensorSuite", content)
        self.assertIn("SafetyWatchdog", content)
        self.assertIn("CommGateway", content)

        # Wire canonical unit into CONOPS.md and report to assert 1:1 parity
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            reports_dir = os.path.join(tmpdir, "docs", "reports")
            os.makedirs(conops_dir, exist_ok=True)
            os.makedirs(reports_dir, exist_ok=True)

            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(content)
            with open(os.path.join(reports_dir, "PHASE_1_EXECUTIVE_ENGINEERING_DELIVERABLES.md"), "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo)
            self.assertEqual(findings, [])

    def test_detects_missing_node(self):
        """Verify failure when a node (e.g. SEN) is omitted in executive deliverables."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            reports_dir = os.path.join(tmpdir, "docs", "reports")
            os.makedirs(conops_dir, exist_ok=True)
            os.makedirs(reports_dir, exist_ok=True)

            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_CONOPS_SV1)

            # Report missing SEN node
            report_missing_node = SAMPLE_DELIVERABLES_MATCHING.replace(
                'SEN["SensorSuite<br/>• sensorDataOut (OUT: SensorStream)"]\n', ''
            )

            with open(os.path.join(reports_dir, "PHASE_1_EXECUTIVE_ENGINEERING_DELIVERABLES.md"), "w", encoding="utf-8") as f:
                f.write(report_missing_node)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo)
            self.assertTrue(len(findings) >= 1)
            self.assertTrue(any("Missing node 'SEN'" in str(f) for f in findings))

    def test_detects_extra_undeclared_node(self):
        """Verify failure when an extra phantom node is present in executive deliverables."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            reports_dir = os.path.join(tmpdir, "docs", "reports")
            os.makedirs(conops_dir, exist_ok=True)
            os.makedirs(reports_dir, exist_ok=True)

            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_CONOPS_SV1)

            # Report contains extra node EXTRA
            report_extra = SAMPLE_DELIVERABLES_MATCHING.replace(
                'ACT["ActuatorSubsystem<br/>• actuatorCmdIn (IN: ActuatorCommand)"]',
                'ACT["ActuatorSubsystem<br/>• actuatorCmdIn (IN: ActuatorCommand)"]\n        EXTRA["ExtraPhantomNode"]'
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

            # Remove watchdogPulseOut port from CC
            report_missing_port = SAMPLE_DELIVERABLES_MATCHING.replace(
                '<br/>• watchdogPulseOut (OUT: HeartbeatPulse)', ''
            )

            with open(os.path.join(reports_dir, "PHASE_1_EXECUTIVE_ENGINEERING_DELIVERABLES.md"), "w", encoding="utf-8") as f:
                f.write(report_missing_port)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo)
            self.assertTrue(len(findings) >= 1)
            self.assertTrue(any("missing port 'watchdogpulseout'" in str(f).lower() for f in findings))

    def test_detects_modified_port_attribute(self):
        """Verify failure when port direction or data type is altered in executive report."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            reports_dir = os.path.join(tmpdir, "docs", "reports")
            os.makedirs(conops_dir, exist_ok=True)
            os.makedirs(reports_dir, exist_ok=True)

            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_CONOPS_SV1)

            # Alter actuatorCmdIn direction from IN to OUT
            report_modified_port = SAMPLE_DELIVERABLES_MATCHING.replace(
                'actuatorCmdIn (IN: ActuatorCommand)',
                'actuatorCmdIn (OUT: ActuatorCommand)'
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
                'CC -->|"CONN-03: HeartbeatPulse (10 Hz Pulse)"| WD\n', ''
            )

            with open(os.path.join(reports_dir, "PHASE_1_EXECUTIVE_ENGINEERING_DELIVERABLES.md"), "w", encoding="utf-8") as f:
                f.write(report_missing_edge)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo)
            self.assertTrue(len(findings) >= 1)
            self.assertTrue(any("Missing connection link 'CC' -> 'WD'" in str(f) for f in findings))

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
                'CONN-03: HeartbeatPulse (10 Hz Pulse)',
                'CONN-99: HeartbeatPulse (10 Hz Pulse)'
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

            # Rename OperationalSegment to AirborneSubsystem
            report_mismatched_sg = SAMPLE_DELIVERABLES_MATCHING.replace(
                'subgraph OperationalSegment',
                'subgraph AirborneSubsystem'
            )

            with open(os.path.join(reports_dir, "PHASE_1_EXECUTIVE_ENGINEERING_DELIVERABLES.md"), "w", encoding="utf-8") as f:
                f.write(report_mismatched_sg)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo)
            self.assertTrue(len(findings) >= 1)
            self.assertTrue(any("Missing subgraph 'OperationalSegment'" in str(f) for f in findings))

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
            scenario_text, "docs/conops/CONOPS.md", target_diagrams={"SV-1"}, allow_partial_snippets=True
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

    # =========================================================================
    # DoDAF SV-4 / OMG UAF Rs-Fn Function-to-Subsystem Allocation Tests (refs #346)
    # =========================================================================

    def test_sv4_registered_in_canonical_diagrams(self):
        """Verify SV-4 is registered in CANONICAL_DIAGRAMS with diagram_num 12."""
        self.assertIn("SV-4", CANONICAL_DIAGRAMS)
        spec = CANONICAL_DIAGRAMS["SV-4"]
        self.assertEqual(spec["diagram_num"], 12)
        self.assertEqual(spec["name"], "Function-to-Subsystem Allocation Diagram")
        self.assertEqual(spec["viewpoint"], "System Viewpoint (SV)")
        self.assertEqual(spec["expected_types"], {"graph", "flowchart"})
        self.assertEqual(spec["min_nodes"], 3)
        self.assertTrue(spec["requires_subgraphs"])
        self.assertIn("DoDAF 2.02 / OMG UAF v2.0 Rs-Fn", spec["standards"])

    def test_sv4_conforming_diagram_passes_gate30_validation(self):
        """Verify conforming SV-4 diagram passes Gate 30 validation with 0 findings."""
        sv4_text = """# Concept of Operations (ConOps)

### 4.6 DoDAF SV-4 / UAF Rs-Fn Function-to-Subsystem Allocation Architecture
In accordance with DoDAF 2.02 SV-4 and OMG UAF v2.0 Rs-Fn, system functions are allocated to subsystems.

```mermaid
flowchart TD
    subgraph CoreController ["Core Real-Time Controller Node"]
        direction TB
        FN01["FN-01: Autonomous Trajectory<br/>Guidance & State Estimation"]
        FN02["FN-02: Actuator Drive<br/>Command Synthesis"]
    end

    subgraph SensorSuite ["Sensor Suite & Edge Processor Node"]
        direction TB
        FN03["FN-03: Multi-Modal Data<br/>Acquisition & Filtering"]
        FN04["FN-04: Environmental Boundary<br/>Limit Monitoring"]
    end

    subgraph SafetyWatchdog ["Independent Hardware Safety Watchdog"]
        direction TB
        FN05["FN-05: Continuous Hardware Pulse<br/>& Interlock Validation"]
        FN06["FN-06: Deterministic Failsafe<br/>Containment Execution"]
    end

    subgraph CommGateway ["PACE Communications Gateway Node"]
        direction TB
        FN07["FN-07: Bidirectional Telemetry<br/>Serialization & Encryption"]
    end

    FN07 -->|"CONN-01: Uplink Command Vector"| FN01
    FN03 -->|"CONN-02: Filtered State Telemetry"| FN01
    FN01 -->|"CONN-03: Actuation Demand"| FN02
    FN04 -->|"CONN-04: Boundary Limit Alert"| FN06
    FN05 -->|"CONN-05: Pulse Loss Interlock"| FN06
    FN06 -->|"CONN-06: Failsafe Override"| FN02
    FN01 -->|"CONN-07: Health & State Stream"| FN07
```
"""
        findings = self.validator.validate_markdown_content(
            sv4_text, "docs/conops/CONOPS.md", target_diagrams={"SV-4"}, allow_partial_snippets=True
        )
        self.assertEqual(findings, [])

    def test_sv4_degenerate_lacking_subgraphs_flagged(self):
        """Verify SV-4 diagram lacking subsystem performer subgraphs is flagged as incomplete."""
        sv4_text = """# Concept of Operations

### 4.6 DoDAF SV-4 Function-to-Subsystem Allocation Architecture

```mermaid
flowchart TD
    FN01["FN-01: Autonomous Guidance"] --> FN02["FN-02: Drive Command"]
    FN02 --> FN03["FN-03: Sensor Processing"]
```
"""
        findings = self.validator.validate_markdown_content(
            sv4_text, "docs/conops/CONOPS.md", target_diagrams={"SV-4"}, allow_partial_snippets=True
        )
        incomplete = [f for f in findings if f.rule_id == RULE_INCOMPLETE_DIAGRAM]
        self.assertTrue(len(incomplete) >= 1)
        self.assertTrue(any("lacks subsystem performer subgraphs" in str(f) for f in incomplete))

    def test_sv4_degenerate_fewer_than_min_nodes_flagged(self):
        """Verify SV-4 diagram with fewer than 3 nodes is flagged as degenerate."""
        sv4_text = """# Concept of Operations

### 4.6 DoDAF SV-4 Function-to-Subsystem Allocation Architecture

```mermaid
flowchart TD
    subgraph CoreController ["Core Controller Node"]
        FN01["FN-01: Autonomous Guidance"] --> FN02["FN-02: Drive Command"]
    end
```
"""
        findings = self.validator.validate_markdown_content(
            sv4_text, "docs/conops/CONOPS.md", target_diagrams={"SV-4"}, allow_partial_snippets=True
        )
        incomplete = [f for f in findings if f.rule_id == RULE_INCOMPLETE_DIAGRAM]
        self.assertTrue(len(incomplete) >= 1)
        self.assertTrue(any("degenerate" in str(f) for f in incomplete))

    def test_sv4_invalid_diagram_type_flagged(self):
        """Verify SV-4 diagram declared as invalid diagram kind (e.g. sequenceDiagram) is flagged."""
        sv4_text = """# Concept of Operations

### 4.6 DoDAF SV-4 Function-to-Subsystem Allocation Architecture

```mermaid
sequenceDiagram
    participant Controller as "Controller"
    participant Sensor as "Sensor Suite"
    participant Actuator as "Actuator"
    Controller ->> Sensor: Poll
    Sensor -->> Controller: Data
    Controller ->> Actuator: Drive
```
"""
        findings = self.validator.validate_markdown_content(
            sv4_text, "docs/conops/CONOPS.md", target_diagrams={"SV-4"}, allow_partial_snippets=True
        )
        incomplete = [f for f in findings if f.rule_id == RULE_INCOMPLETE_DIAGRAM]
        self.assertTrue(len(incomplete) >= 1)
        self.assertTrue(any("declaration 'sequencediagram' invalid" in str(f) for f in incomplete))

    def test_sv4_missing_functional_nodes_flagged(self):
        """Verify SV-4 diagram lacking functional activity/operation nodes is flagged."""
        sv4_text = """# Concept of Operations

### 4.6 DoDAF SV-4 Function-to-Subsystem Allocation Architecture

```mermaid
flowchart TD
    subgraph ControllerNode ["Controller Subsystem Node"]
        P1["Physical Port 1"]
        P2["Physical Port 2"]
        P3["Physical Port 3"]
    end
    P1 --> P2 --> P3
```
"""
        findings = self.validator.validate_markdown_content(
            sv4_text, "docs/conops/CONOPS.md", target_diagrams={"SV-4"}, allow_partial_snippets=True
        )
        incomplete = [f for f in findings if f.rule_id == RULE_INCOMPLETE_DIAGRAM]
        self.assertTrue(len(incomplete) >= 1)
        self.assertTrue(any("lacks functional activity or operation nodes" in str(f) for f in incomplete))

    def test_sv4_structural_heuristic_recognition(self):
        """Verify SV-4 diagram is recognized structurally via subsystem subgraphs + functional nodes."""
        text = """# System Operational Decomposition

### Architecture Allocation Matrix

```mermaid
flowchart TD
    subgraph ControllerNode ["Core Controller Subsystem"]
        FN01["FN-01: Guidance Law Synthesis"]
        FN02["FN-02: Actuation Output"]
    end
    subgraph EdgeProcessor ["Edge Sensor Processor"]
        FN03["FN-03: Measurement Filtering"]
    end
    FN03 --> FN01 --> FN02
```
"""
        blocks, _ = _extract_blocks_and_syntax_findings(text, "CONOPS.md")
        self.assertEqual(len(blocks), 1)
        self.assertTrue(_matches_canonical_diagram("SV-4", blocks[0]))

    def test_sv4_missing_in_downstream_workspace_flagged(self):
        """Verify validate() flags missing SV-4 diagram in downstream workspace with active specs."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(conops_dir, exist_ok=True)
            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write("""# Concept of Operations

### 1.1 System Context (OV-1)
```mermaid
flowchart TD
    Actor["External Operator"] --> System["Core System Boundary"]
```
""")
            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo)
            missing_diagrams = [
                f.detail.get("diagram_id") for f in findings if f.rule_id == RULE_DIAGRAM_MISSING
            ]
            self.assertIn("SV-4", missing_diagrams)

    def test_sv4_upstream_clean_landing_zone_graceful_pass(self):
        """Verify validate() gracefully passes on upstream template clean landing zones."""
        with tempfile.TemporaryDirectory() as tmpdir:
            upstream_marker = os.path.join(tmpdir, ".pipeline", "upstream")
            os.makedirs(upstream_marker, exist_ok=True)
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(conops_dir, exist_ok=True)
            with open(os.path.join(conops_dir, ".gitkeep"), "w") as f:
                f.write("")
            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo)
            self.assertEqual(findings, [])

    # =========================================================================
    # DoDAF OV-5a / OMG UAF Op-Tx Operational Activity Tree Tests (refs #348)
    # =========================================================================

    def test_ov5a_registered_in_canonical_diagrams(self):
        """Verify OV-5a is registered in CANONICAL_DIAGRAMS with diagram_num 13."""
        self.assertIn("OV-5a", CANONICAL_DIAGRAMS)
        spec = CANONICAL_DIAGRAMS["OV-5a"]
        self.assertEqual(spec["diagram_num"], 13)
        self.assertEqual(spec["name"], "Operational Activity Decomposition Tree")
        self.assertEqual(spec["viewpoint"], "Operational Viewpoint (OV)")
        self.assertEqual(spec["expected_types"], {"graph", "flowchart", "mindmap"})
        self.assertEqual(spec["min_nodes"], 4)
        self.assertIn("DoDAF 2.02 / OMG UAF v2.0 Op-Tx / ISO/IEC/IEEE 29148 §6.4.3", spec["standards"])

    def test_ov5a_conforming_flowchart_passes_gate30_validation(self):
        """Verify conforming OV-5a flowchart passes Gate 30 validation with 0 findings."""
        ov5a_text = """# Concept of Operations (ConOps)

### 6.1.1 DoDAF OV-5a / UAF Op-Tx Operational Activity Decomposition Tree
Hierarchical decomposition of operational mission activities into sub-activities.

```mermaid
flowchart TD
    MissionOperations["Autonomous Mission Operations"]

    PhasePreOp["Pre-Operation & Health"]
    PhaseTransit["Corridor Transit & Guidance"]
    PhaseMission["Mission Execution & Sensing"]
    PhaseContingency["Contingency & Containment"]
    PhaseShutdown["Recovery & Secure Shutdown"]

    OA01["OA-01: Pre-Operation<br/>Health Verification"]
    OA02["OA-02: Transit &<br/>Corridor Execution"]
    OA03["OA-03: Primary<br/>Operational Processing"]
    OA04["OA-04: Supervisory<br/>Telemetry Exchange"]
    OA05["OA-05: Environmental<br/>Limit Monitoring"]
    OA06["OA-06: Energy Resource<br/>Management"]
    OA07["OA-07: Contingency<br/>Containment Execution"]
    OA08["OA-08: Post-Operation<br/>Secure Shutdown"]

    MissionOperations --> PhasePreOp
    MissionOperations --> PhaseTransit
    MissionOperations --> PhaseMission
    MissionOperations --> PhaseContingency
    MissionOperations --> PhaseShutdown

    PhasePreOp --> OA01
    PhaseTransit --> OA02
    PhaseMission --> OA03
    PhaseMission --> OA04
    PhaseMission --> OA05
    PhaseMission --> OA06
    PhaseContingency --> OA07
    PhaseShutdown --> OA08
```
"""
        findings = self.validator.validate_markdown_content(
            ov5a_text, "docs/conops/CONOPS.md", target_diagrams={"OV-5a"}, allow_partial_snippets=True
        )
        self.assertEqual(findings, [])

    def test_ov5a_conforming_mindmap_passes_gate30_validation(self):
        """Verify conforming OV-5a mindmap passes Gate 30 validation with 0 findings."""
        ov5a_mindmap_text = """# Concept of Operations (ConOps)

### 6.1.1 DoDAF OV-5a / UAF Op-Tx Operational Activity Decomposition Tree

```mermaid
mindmap
  root((Autonomous Mission Operations))
    PhasePreOp[Pre-Operation & Health]
      OA01[OA-01: Pre-Operation Health Verification]
    PhaseTransit[Corridor Transit & Guidance]
      OA02[OA-02: Transit & Corridor Execution]
    PhaseMission[Mission Execution & Sensing]
      OA03[OA-03: Primary Operational Processing]
      OA04[OA-04: Supervisory Telemetry Exchange]
      OA05[OA-05: Environmental Limit Monitoring]
      OA06[OA-06: Energy Resource Management]
    PhaseContingency[Contingency & Containment]
      OA07[OA-07: Contingency Containment Execution]
    PhaseShutdown[Recovery & Secure Shutdown]
      OA08[OA-08: Post-Operation Secure Shutdown]
```
"""
        findings = self.validator.validate_markdown_content(
            ov5a_mindmap_text, "docs/conops/CONOPS.md", target_diagrams={"OV-5a"}, allow_partial_snippets=True
        )
        self.assertEqual(findings, [])

    def test_ov5a_degenerate_fewer_than_min_nodes_flagged(self):
        """Verify OV-5a diagram with fewer than 4 nodes is flagged as degenerate."""
        ov5a_text = """# Concept of Operations

### 6.1.1 DoDAF OV-5a Operational Activity Decomposition Tree

```mermaid
flowchart TD
    Mission["Autonomous Mission Operations"] --> Phase["Pre-Operation"]
    Phase --> OA01["OA-01: Health Verification"]
```
"""
        findings = self.validator.validate_markdown_content(
            ov5a_text, "docs/conops/CONOPS.md", target_diagrams={"OV-5a"}, allow_partial_snippets=True
        )
        incomplete = [f for f in findings if f.rule_id == RULE_INCOMPLETE_DIAGRAM]
        self.assertTrue(len(incomplete) >= 1)
        self.assertTrue(any("degenerate" in str(f) and "at least 4 connected nodes" in str(f) for f in incomplete))

    def test_ov5a_degenerate_missing_oa_nodes_flagged(self):
        """Verify OV-5a diagram lacking OA- nodes is flagged as incomplete."""
        ov5a_text = """# Concept of Operations

### 6.1.1 DoDAF OV-5a Operational Activity Decomposition Tree

```mermaid
flowchart TD
    Mission["Autonomous Mission Operations"]
    Phase1["Phase 1: Startup"]
    Phase2["Phase 2: Execution"]
    Phase3["Phase 3: Shutdown"]

    Mission --> Phase1
    Mission --> Phase2
    Mission --> Phase3
```
"""
        findings = self.validator.validate_markdown_content(
            ov5a_text, "docs/conops/CONOPS.md", target_diagrams={"OV-5a"}, allow_partial_snippets=True
        )
        incomplete = [f for f in findings if f.rule_id == RULE_INCOMPLETE_DIAGRAM]
        self.assertTrue(len(incomplete) >= 1)
        self.assertTrue(any("lacks operational activity leaf nodes" in str(f) for f in incomplete))

    def test_ov5a_degenerate_linear_chain_without_branching_flagged(self):
        """Verify OV-5a diagram configured as a linear chain without fan-out branching is flagged."""
        ov5a_text = """# Concept of Operations

### 6.1.1 DoDAF OV-5a Operational Activity Decomposition Tree

```mermaid
flowchart TD
    Mission["Autonomous Mission Operations"] --> Phase["Phase Execution"]
    Phase --> OA01["OA-01: Health Verification"]
    OA01 --> OA02["OA-02: Guidance Law"]
```
"""
        findings = self.validator.validate_markdown_content(
            ov5a_text, "docs/conops/CONOPS.md", target_diagrams={"OV-5a"}, allow_partial_snippets=True
        )
        incomplete = [f for f in findings if f.rule_id == RULE_INCOMPLETE_DIAGRAM]
        self.assertTrue(len(incomplete) >= 1)
        self.assertTrue(any("linear chain detected without parent-child fan-out" in str(f) for f in incomplete))

    def test_ov5a_degenerate_insufficient_edges_flagged(self):
        """Verify OV-5a diagram with fewer than 3 edges is flagged."""
        ov5a_text = """# Concept of Operations

### 6.1.1 DoDAF OV-5a Operational Activity Decomposition Tree

```mermaid
flowchart TD
    Mission["Autonomous Mission Operations"]
    OA01["OA-01: Health Verification"]
    OA02["OA-02: Guidance Law"]
    OA03["OA-03: Sensor Processing"]

    Mission --> OA01
    Mission --> OA02
```
"""
        findings = self.validator.validate_markdown_content(
            ov5a_text, "docs/conops/CONOPS.md", target_diagrams={"OV-5a"}, allow_partial_snippets=True
        )
        incomplete = [f for f in findings if f.rule_id == RULE_INCOMPLETE_DIAGRAM]
        self.assertTrue(len(incomplete) >= 1)
        self.assertTrue(any("lacks hierarchical tree branching" in str(f) for f in incomplete))

    def test_ov5a_invalid_diagram_type_flagged(self):
        """Verify OV-5a declared as invalid kind (e.g. sequenceDiagram) is flagged."""
        ov5a_text = """# Concept of Operations

### 6.1.1 DoDAF OV-5a Operational Activity Decomposition Tree

```mermaid
sequenceDiagram
    participant Mission as "Autonomous Mission Operations"
    participant Phase as "Pre-Operation"
    participant OA01 as "OA-01"
    participant OA02 as "OA-02"
    Mission ->> Phase: Init
    Phase ->> OA01: Exec
    Phase ->> OA02: Exec
```
"""
        findings = self.validator.validate_markdown_content(
            ov5a_text, "docs/conops/CONOPS.md", target_diagrams={"OV-5a"}, allow_partial_snippets=True
        )
        incomplete = [f for f in findings if f.rule_id == RULE_INCOMPLETE_DIAGRAM]
        self.assertTrue(len(incomplete) >= 1)
        self.assertTrue(any("declaration 'sequencediagram' invalid" in str(f) for f in incomplete))

    def test_ov5a_structural_heuristic_recognition(self):
        """Verify OV-5a diagram is recognized structurally via mission root and OA- nodes."""
        text = """# System Operational Decomposition

### Mission Activity Tree

```mermaid
flowchart TD
    MissionOperations["Autonomous Mission Operations"]
    Phase1["Phase Guidance"]
    OA01["OA-01: Health Verification"]
    OA02["OA-02: Trajectory Guidance"]
    OA03["OA-03: Sensor Fusion"]

    MissionOperations --> Phase1
    MissionOperations --> OA01
    Phase1 --> OA02
    Phase1 --> OA03
```
"""
        blocks, _ = _extract_blocks_and_syntax_findings(text, "CONOPS.md")
        self.assertEqual(len(blocks), 1)
        self.assertTrue(_matches_canonical_diagram("OV-5a", blocks[0]))

    def test_ov5a_missing_in_downstream_workspace_flagged(self):
        """Verify validate() flags missing OV-5a diagram in downstream workspace with active specs."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(conops_dir, exist_ok=True)
            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write("""# Concept of Operations

### 1.1 System Context (OV-1)
```mermaid
flowchart TD
    Actor["External Operator"] --> System["Core System Boundary"]
```
""")
            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo)
            missing_diagrams = [
                f.detail.get("diagram_id") for f in findings if f.rule_id == RULE_DIAGRAM_MISSING
            ]
            self.assertIn("OV-5a", missing_diagrams)

    def test_ov5a_upstream_clean_landing_zone_graceful_pass(self):
        """Verify validate() gracefully passes on upstream template clean landing zones."""
        with tempfile.TemporaryDirectory() as tmpdir:
            upstream_marker = os.path.join(tmpdir, ".pipeline", "upstream")
            os.makedirs(upstream_marker, exist_ok=True)
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(conops_dir, exist_ok=True)
            with open(os.path.join(conops_dir, ".gitkeep"), "w") as f:
                f.write("")
            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo)
            self.assertEqual(findings, [])

    # =========================================================================
    # DoDAF OV-4 / OMG UAF Op-Or Organizational Command Hierarchy Tests (refs #345)
    # =========================================================================

    def test_ov4_canonical_diagram_registered(self):
        """Verify OV-4 is registered in CANONICAL_DIAGRAMS with diagram_num 14."""
        self.assertIn("OV-4", CANONICAL_DIAGRAMS)
        spec = CANONICAL_DIAGRAMS["OV-4"]
        self.assertEqual(spec["diagram_num"], 14)
        self.assertEqual(spec["name"], "Organizational Command & Authority Hierarchy View")
        self.assertEqual(spec["viewpoint"], "Operational Viewpoint (OV)")
        self.assertEqual(spec["expected_types"], {"graph", "flowchart"})
        self.assertEqual(spec["min_nodes"], 4)
        self.assertIn("DoDAF 2.02 / OMG UAF v2.0 Op-Or / ISO/IEC/IEEE 29148 §6.4.3", spec["standards"])
        self.assertEqual(spec["default_location"], "docs/conops/CONOPS.md")

    def test_ov4_canonical_diagram_passes_gate30(self):
        """Verify conforming OV-4 flowchart TD passes Gate 30 validation with 0 findings."""
        ov4_text = """# Concept of Operations (ConOps)

### 1.6.1 DoDAF OV-4 / OMG UAF Op-Or Organizational Command & Authority Hierarchy
Organizational command structure and reporting paths across operational roles.

```mermaid
flowchart TD
    Director["Executive Authority<br/>(Operations Director)"]
    Supervisor["Mission Supervisor<br/>(UCL-03)"]
    Operator["System Operator<br/>(UCL-01)"]
    PayloadSpec["Payload Specialist<br/>(UCL-02)"]
    SafetyMon["Safety Monitor<br/>(UCL-05)"]
    SupportOrg["Support Organization<br/>(Maintenance Depot)"]
    Technician["Maintenance Technician<br/>(UCL-04)"]

    Director -->|"Operational Directive"| Supervisor
    Supervisor -->|"Command Authority"| Operator
    Supervisor -->|"Payload Tasking"| PayloadSpec
    Supervisor -->|"Safety Coordination"| SafetyMon
    SupportOrg -->|"Logistics Oversight"| Technician
    Technician -.->|"Readiness Status"| Supervisor
    Operator -.->|"Operational Status"| Supervisor
```
"""
        findings = self.validator.validate_markdown_content(
            ov4_text, "docs/conops/CONOPS.md", target_diagrams={"OV-4"}, allow_partial_snippets=True
        )
        self.assertEqual(findings, [])

    def test_ov4_degenerate_fewer_than_min_nodes_flagged(self):
        """Verify OV-4 diagram with fewer than 4 nodes is flagged as degenerate."""
        ov4_text = """# Concept of Operations

### 1.6.1 DoDAF OV-4 Command and Authority Hierarchy

```mermaid
flowchart TD
    Director["Operations Director"] --> Supervisor["Mission Supervisor UCL-03"]
    Supervisor --> Operator["System Operator UCL-01"]
```
"""
        findings = self.validator.validate_markdown_content(
            ov4_text, "docs/conops/CONOPS.md", target_diagrams={"OV-4"}, allow_partial_snippets=True
        )
        incomplete = [f for f in findings if f.rule_id == RULE_INCOMPLETE_DIAGRAM]
        self.assertTrue(len(incomplete) >= 1)
        self.assertTrue(any("degenerate" in str(f) and "at least 4 connected nodes" in str(f) for f in incomplete))

    def test_ov4_degenerate_missing_operator_nodes_flagged(self):
        """Verify OV-4 diagram lacking operator/supervisor/role nodes is flagged as incomplete."""
        ov4_text = """# Concept of Operations

### 1.6.1 DoDAF OV-4 Command and Authority Hierarchy

```mermaid
flowchart TD
    NodeA["Server Cluster Alpha"]
    NodeB["Server Cluster Beta"]
    NodeC["Storage Node Gamma"]
    NodeD["Network Switch Delta"]

    NodeA --> NodeB
    NodeB --> NodeC
    NodeC --> NodeD
```
"""
        findings = self.validator.validate_markdown_content(
            ov4_text, "docs/conops/CONOPS.md", target_diagrams={"OV-4"}, allow_partial_snippets=True
        )
        incomplete = [f for f in findings if f.rule_id == RULE_INCOMPLETE_DIAGRAM]
        self.assertTrue(len(incomplete) >= 1)
        self.assertTrue(any("lacks organizational role or operator nodes" in str(f) for f in incomplete))

    def test_ov4_degenerate_insufficient_edges_flagged(self):
        """Verify OV-4 diagram with fewer than 3 edges is flagged."""
        ov4_text = """# Concept of Operations

### 1.6.1 DoDAF OV-4 Command and Authority Hierarchy

```mermaid
flowchart TD
    Director["Operations Director"]
    Supervisor["Mission Supervisor UCL-03"]
    Operator["System Operator UCL-01"]
    Technician["Maintenance Technician UCL-04"]

    Director --> Supervisor
    Supervisor --> Operator
```
"""
        findings = self.validator.validate_markdown_content(
            ov4_text, "docs/conops/CONOPS.md", target_diagrams={"OV-4"}, allow_partial_snippets=True
        )
        incomplete = [f for f in findings if f.rule_id == RULE_INCOMPLETE_DIAGRAM]
        self.assertTrue(len(incomplete) >= 1)
        self.assertTrue(any("lacks hierarchical command and reporting relationships" in str(f) for f in incomplete))

    def test_ov4_invalid_diagram_type_flagged(self):
        """Verify OV-4 declared as invalid kind (e.g. sequenceDiagram) is flagged."""
        ov4_text = """# Concept of Operations

### 1.6.1 DoDAF OV-4 Command and Authority Hierarchy

```mermaid
sequenceDiagram
    participant Director as "Operations Director"
    participant Supervisor as "Mission Supervisor UCL-03"
    participant Operator as "System Operator UCL-01"
    participant Technician as "Maintenance Technician UCL-04"
    Director ->> Supervisor: Directive
    Supervisor ->> Operator: Command
    Technician ->> Supervisor: Status
```
"""
        findings = self.validator.validate_markdown_content(
            ov4_text, "docs/conops/CONOPS.md", target_diagrams={"OV-4"}, allow_partial_snippets=True
        )
        incomplete = [f for f in findings if f.rule_id == RULE_INCOMPLETE_DIAGRAM]
        self.assertTrue(len(incomplete) >= 1)
        self.assertTrue(any("declaration 'sequencediagram' invalid" in str(f) for f in incomplete))

    def test_ov4_structural_heuristic_recognition(self):
        """Verify OV-4 diagram is recognized structurally via role nodes and command hierarchy terms."""
        text = """# Operational Management Architecture

### Command and Authority Hierarchy

```mermaid
flowchart TD
    Director["Operations Director"]
    Supervisor["Mission Supervisor UCL-03"]
    Operator["System Operator UCL-01"]
    Technician["Maintenance Technician UCL-04"]

    Director -->|"Strategic Directive"| Supervisor
    Supervisor -->|"Supervisory Command"| Operator
    Technician -.->|"Readiness Status"| Supervisor
```
"""
        blocks, _ = _extract_blocks_and_syntax_findings(text, "CONOPS.md")
        self.assertEqual(len(blocks), 1)
        self.assertTrue(_matches_canonical_diagram("OV-4", blocks[0]))

    def test_ov4_missing_in_downstream_workspace_flagged(self):
        """Verify validate() flags missing OV-4 diagram in downstream workspace with active specs."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(conops_dir, exist_ok=True)
            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write("""# Concept of Operations

### 1.1 System Context (OV-1)
```mermaid
flowchart TD
    Actor["External Operator"] --> System["Core System Boundary"]
```
""")
            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo)
            missing_diagrams = [
                f.detail.get("diagram_id") for f in findings if f.rule_id == RULE_DIAGRAM_MISSING
            ]
            self.assertIn("OV-4", missing_diagrams)

    def test_ov4_upstream_clean_landing_zone_graceful_pass(self):
        """Verify validate() gracefully passes on upstream template clean landing zones."""
        with tempfile.TemporaryDirectory() as tmpdir:
            upstream_marker = os.path.join(tmpdir, ".pipeline", "upstream")
            os.makedirs(upstream_marker, exist_ok=True)
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(conops_dir, exist_ok=True)
            with open(os.path.join(conops_dir, ".gitkeep"), "w") as f:
                f.write("")
            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo)
            self.assertEqual(findings, [])

    # =========================================================================
    # SPATIAL-4D / Environmental Viewpoint & Canonical ConOps Parity Tests (refs #339, #351)
    # =========================================================================

    def test_spatial4d_registered_in_canonical_diagrams(self):
        """Verify SPATIAL-4D is registered in CANONICAL_DIAGRAMS with diagram_num 11."""
        self.assertIn("SPATIAL-4D", CANONICAL_DIAGRAMS)
        self.assertNotIn("SORA-4D", CANONICAL_DIAGRAMS)
        spec = CANONICAL_DIAGRAMS["SPATIAL-4D"]
        self.assertEqual(spec["diagram_num"], 11)
        self.assertEqual(spec["name"], "4D Spatial Volume & Environmental Containment Diagram")
        self.assertEqual(spec["viewpoint"], "Spatial / Environmental Viewpoint (SPATIAL)")
        self.assertEqual(spec["expected_types"], {"graph", "flowchart"})
        self.assertEqual(spec["min_nodes"], 2)
        self.assertIn("ISO/IEC/IEEE 15288 §6.4.4 / MIL-STD-882E", spec["standards"])
        self.assertEqual(spec["default_location"], "docs/conops/CONOPS.md")

    def test_conops_canonical_diagrams_registry(self):
        """Verify CONOPS_CANONICAL_DIAGRAMS registry contains exactly the 11 ConOps canonical viewpoints."""
        expected = {
            "OV-1", "OV-2", "OV-4", "OV-5a", "OV-5b", "OV-6c",
            "SV-1", "SV-2", "SV-4", "SV-10b", "SPATIAL-4D"
        }
        self.assertEqual(CONOPS_CANONICAL_DIAGRAMS, expected)
        self.assertEqual(len(CONOPS_CANONICAL_DIAGRAMS), 11)

    def test_spatial4d_matching_and_sora4d_alias(self):
        """Verify _matches_canonical_diagram recognizes both SPATIAL-4D and backwards-compatible SORA-4D alias."""
        text_spatial = """# Spatial Containment
### 5.5 DoDAF / ISO 15288 SPATIAL-4D 4D Spatial Volume & Environmental Containment Diagram
```mermaid
flowchart TB
    A["Operational Volume"] --> B["Contingency Buffer"]
```
"""
        blocks_spatial, _ = _extract_blocks_and_syntax_findings(text_spatial, "CONOPS.md")
        self.assertEqual(len(blocks_spatial), 1)
        self.assertTrue(_matches_canonical_diagram("SPATIAL-4D", blocks_spatial[0]))
        # Also matches when queried via legacy key
        self.assertTrue(_matches_canonical_diagram("SORA-4D", blocks_spatial[0]))

        text_sora_legacy = """# Risk Containment
### 5.5 SORA-4D Ground Risk Buffer Containment Topology
```mermaid
flowchart TB
    A["Flight Geography"] --> B["Ground Risk Buffer"]
```
"""
        blocks_sora, _ = _extract_blocks_and_syntax_findings(text_sora_legacy, "CONOPS.md")
        self.assertEqual(len(blocks_sora), 1)
        self.assertTrue(_matches_canonical_diagram("SPATIAL-4D", blocks_sora[0]))
        self.assertTrue(_matches_canonical_diagram("SORA-4D", blocks_sora[0]))

    def test_validate_markdown_content_allow_partial_snippets(self):
        """Verify allow_partial_snippets=True avoids flagging RULE_DIAGRAM_MISSING."""
        text = """# Single Viewpoint Snippet
### 1.1 System Context (OV-1)
```mermaid
flowchart TD
    Actor["Operator"] --> System["System"]
```
"""
        findings_strict = self.validator.validate_markdown_content(
            text, "docs/conops/CONOPS.md", allow_partial_snippets=False
        )
        missing_strict = [f for f in findings_strict if f.rule_id == RULE_DIAGRAM_MISSING]
        self.assertTrue(len(missing_strict) > 0)

        findings_partial = self.validator.validate_markdown_content(
            text, "docs/conops/CONOPS.md", allow_partial_snippets=True
        )
        missing_partial = [f for f in findings_partial if f.rule_id == RULE_DIAGRAM_MISSING]
        self.assertEqual(len(missing_partial), 0)

    def test_validate_markdown_content_conops_unblinded_evaluates_only_conops_diagrams(self):
        """Verify ConOps file content evaluates only CONOPS_CANONICAL_DIAGRAMS and not ICD or STPA."""
        text = """# ConOps Overview
### 1.1 System Context (OV-1)
```mermaid
flowchart TD
    Actor["Operator"] --> System["System"]
```
"""
        findings = self.validator.validate_markdown_content(
            text, "docs/conops/CONOPS.md", allow_partial_snippets=False
        )
        missing_diagram_ids = {
            f.detail.get("diagram_id") for f in findings if f.rule_id == RULE_DIAGRAM_MISSING
        }
        self.assertIn("SPATIAL-4D", missing_diagram_ids)
        self.assertIn("SV-1", missing_diagram_ids)
        self.assertNotIn("ICD-N2", missing_diagram_ids)
        self.assertNotIn("STPA-CONTROL", missing_diagram_ids)
        self.assertNotIn("STPA-RTA", missing_diagram_ids)


if __name__ == "__main__":
    unittest.main()

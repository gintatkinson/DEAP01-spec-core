"""
Unit and integration tests for Check 27: Executive Deliverable Traceability & Completeness Gate.
Resolves GitHub Issues #283 & #284.

Verifies:
1. Symbol export in parity_auditor.validators and registration in AGGREGATING_VALIDATORS.
2. Rule `executive-table-unanchored-provenance`:
   - Scans markdown tables under `docs/reports/` and `docs/management/` mentioning "Subsystem",
     "Part", or "System Control Action" in header.
   - Passes when tables possess provenance columns or row-level citations to `schema/DEAP_MODEL.sysml`
     (or AST nodes / PartDefs), schema source documents (.md in schema/), or regulatory standards
     (STANAG, MIL-STD, DO-178, DO-254, ARP4754, ARP4761, ISO, IEEE, ASTM).
   - Fails when rows lack SSOT/OEM citations.
3. Rule `executive-diagram-subsystem-incomplete`:
   - Scans Mermaid diagrams under headings matching "subsystem architecture" or "system architecture".
   - Passes when diagram contains `%% Realizes:` and `%% Coverage:` metadata headers.
   - Passes when diagram encompasses all declared AST subsystems in `schema/DEAP_MODEL.sysml`.
   - Passes when truncated diagram contains an explicit scoping rationale.
   - Fails when declared subsystems are omitted without metadata headers or explicit scoping rationale.
4. Upstream clean landing zone passes cleanly (zero findings on empty schema/ or .gitkeep).
5. Integration with scripts/verify_downstream_baseline.py Check 27 gate.
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
from parity_auditor.aggregator import AGGREGATING_VALIDATORS

try:
    from parity_auditor.validators.executive_deliverable_traceability_validator import (
        ExecutiveDeliverableTraceabilityValidator,
        validate_executive_deliverable_traceability,
        RULE_TABLE_UNANCHORED,
        RULE_DIAGRAM_INCOMPLETE,
    )
except ImportError:
    ExecutiveDeliverableTraceabilityValidator = None
    validate_executive_deliverable_traceability = None
    RULE_TABLE_UNANCHORED = "executive-table-unanchored-provenance"
    RULE_DIAGRAM_INCOMPLETE = "executive-diagram-subsystem-incomplete"

try:
    from parity_auditor.validators import (
        ExecutiveDeliverableTraceabilityValidator as ExportedValidator,
    )
except ImportError:
    ExportedValidator = None

try:
    from scripts.verify_downstream_baseline import (
        check_executive_deliverable_traceability,
        check_executive_deliverable_traceability_gate,
    )
except ImportError:
    check_executive_deliverable_traceability = None
    check_executive_deliverable_traceability_gate = None


SAMPLE_SYSML_MODEL = """package AutonomousSystem_SSOT {
    part def FlightControlComputer;
    part def NavigationSubsystem;
    part def ActuationSubsystem;
    part def PayloadSubsystem;
}
"""

SAMPLE_VALID_TABLE_REPORT = """# Executive Deliverables: Phase 1 Synthesis

## Subsystem Allocation Matrix
| Subsystem ID | Subsystem Name | Allocation / Function | SSOT / AST Citation | Regulatory Standard |
| :--- | :--- | :--- | :--- | :--- |
| SUB-01 | Flight Control Computer | Autonomous guidance & flight control | `schema/DEAP_MODEL.sysml#L45` (PartDef FCC) | DO-178C DAL-B |
| SUB-02 | Navigation & Sensing | Inertial estimation & GNSS | [NavSystem](schema/DEAP_MODEL.sysml) | ARP4754A / MIL-STD-882E |
| SUB-03 | Actuation Subsystem | Aerodynamic surface control | `schema/OEM_BOM.md#L12` | STANAG 4586 |
| SUB-04 | Payload Subsystem | Sensor payload management | `schema/DEAP_MODEL.sysml#L90` | ISO 26262 |
"""

SAMPLE_VALID_TABLE_MANAGEMENT = """# WBS Deliverables Suite

## System Control Action Matrix
| Action ID | System Control Action | Responsible Subsystem | Provenance / Authority | Standard |
| :--- | :--- | :--- | :--- | :--- |
| SCA-01 | Arm Propulsion Motor | GroundControlStation | `schema/DEAP_MODEL.sysml#L110` | ASTM F3269-17 |
| SCA-02 | Deploy Parachute | RecoverySubsystem | `schema/DEAP_MODEL.sysml#L140` | MIL-STD-882E |
"""

SAMPLE_UNANCHORED_TABLE_REPORT = """# Executive Deliverables: Phase 1 Synthesis

## Subsystem Allocation Matrix
| Subsystem ID | Subsystem Name | Allocation / Function | Status |
| :--- | :--- | :--- | :--- |
| SUB-01 | Flight Control Computer | Autonomous guidance & flight control | In Progress |
| SUB-02 | Navigation & Sensing | Inertial estimation & GNSS | In Progress |
| SUB-03 | Actuation Subsystem | Aerodynamic surface control | Planned |
"""

SAMPLE_DIAGRAM_WITH_REALIZES_AND_COVERAGE = """# Executive Deliverables: Phase 1 Synthesis

## System Architecture

```mermaid
flowchart TD
    %% Realizes: Subsystem Architecture
    %% Coverage: FlightControlComputer, NavigationSubsystem
    FCC["FlightControlComputer"] --> NAV["NavigationSubsystem"]
```
"""

SAMPLE_DIAGRAM_ENCOMPASSING_ALL_AST = """# Executive Deliverables: Phase 1 Synthesis

## Subsystem Architecture Overview

```mermaid
flowchart TD
    FCC["FlightControlComputer"]
    NAV["NavigationSubsystem"]
    ACT["ActuationSubsystem"]
    PAY["PayloadSubsystem"]
    FCC --> NAV
    FCC --> ACT
    FCC --> PAY
```
"""

SAMPLE_DIAGRAM_TRUNCATED_MISSING_SUBSYSTEMS = """# Executive Deliverables: Phase 1 Synthesis

## Subsystem Architecture Overview

```mermaid
flowchart TD
    FCC["FlightControlComputer"]
    FCC --> EXT["ExternalRadio"]
```
"""

SAMPLE_DIAGRAM_TRUNCATED_WITH_SCOPING_RATIONALE = """# Executive Deliverables: Phase 1 Synthesis

## Subsystem Architecture Overview

> Scoping Rationale: This diagram depicts the primary compute core. Navigation, actuation, and payload subsystems are detailed in ICD documents.

```mermaid
flowchart TD
    FCC["FlightControlComputer"]
    FCC --> EXT["ExternalRadio"]
```
"""


class TestExecutiveDeliverableTraceability(unittest.TestCase):
    def setUp(self):
        if ExecutiveDeliverableTraceabilityValidator is not None:
            self.validator = ExecutiveDeliverableTraceabilityValidator()
        else:
            self.validator = None

    def _require_validator(self):
        if self.validator is None:
            self.fail("ExecutiveDeliverableTraceabilityValidator is not implemented yet (RED phase)")

    def test_registered_in_aggregating_validators(self):
        """Verify validator is registered in AGGREGATING_VALIDATORS."""
        if ExecutiveDeliverableTraceabilityValidator is None:
            self.fail("ExecutiveDeliverableTraceabilityValidator is not implemented yet (RED phase)")
        self.assertIn(ExecutiveDeliverableTraceabilityValidator, AGGREGATING_VALIDATORS)
        self.assertIsNotNone(ExportedValidator)
        self.assertEqual(ExportedValidator, ExecutiveDeliverableTraceabilityValidator)
        if validate_executive_deliverable_traceability is not None:
            self.assertTrue(callable(validate_executive_deliverable_traceability))

    def test_table_provenance_valid(self):
        """Verify tables with SSOT, schema doc, or regulatory citations pass."""
        self._require_validator()
        with tempfile.TemporaryDirectory() as tmpdir:
            reports_dir = os.path.join(tmpdir, "docs", "reports")
            mgmt_dir = os.path.join(tmpdir, "docs", "management")
            os.makedirs(reports_dir, exist_ok=True)
            os.makedirs(mgmt_dir, exist_ok=True)

            with open(os.path.join(reports_dir, "PHASE_1_EXECUTIVE_ENGINEERING_DELIVERABLES.md"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_VALID_TABLE_REPORT)

            with open(os.path.join(mgmt_dir, "WBS_DELIVERABLES_SUITE.md"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_VALID_TABLE_MANAGEMENT)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo)
            table_findings = [f for f in findings if getattr(f, "rule_id", str(f)) == RULE_TABLE_UNANCHORED]
            self.assertEqual(table_findings, [])

    def test_table_unanchored_provenance_failure(self):
        """Verify failure when subsystem/part/control action table lacks SSOT citations."""
        self._require_validator()
        with tempfile.TemporaryDirectory() as tmpdir:
            reports_dir = os.path.join(tmpdir, "docs", "reports")
            os.makedirs(reports_dir, exist_ok=True)

            with open(os.path.join(reports_dir, "PHASE_1_EXECUTIVE_ENGINEERING_DELIVERABLES.md"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_UNANCHORED_TABLE_REPORT)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo)
            table_findings = [f for f in findings if getattr(f, "rule_id", str(f)) == RULE_TABLE_UNANCHORED]
            self.assertTrue(len(table_findings) >= 1)
            self.assertTrue(any(RULE_TABLE_UNANCHORED in str(f) or "unanchored" in str(f).lower() or "citation" in str(f).lower() for f in table_findings))

    def test_table_provenance_downstream_schema_markdown_citation(self):
        """Verify table citing schema source .md documents directly passes."""
        self._require_validator()
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            reports_dir = os.path.join(tmpdir, "docs", "reports")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(reports_dir, exist_ok=True)

            with open(os.path.join(schema_dir, "a5-user-manual-2.md"), "w", encoding="utf-8") as f:
                f.write("# A5 User Manual\n")
            with open(os.path.join(schema_dir, "a5-prep-and-safety-rev7.md"), "w", encoding="utf-8") as f:
                f.write("# A5 Prep and Safety\n")

            content = """# Executive Deliverables: Phase 1 Synthesis

## Subsystem Allocation Matrix
| Subsystem ID | Subsystem Name | Allocation / Function | SSOT / AST Citation |
| :--- | :--- | :--- | :--- |
| SUB-01 | Flight Control Computer | Autonomous guidance & flight control | (`a5-user-manual-2.md` §7.3) |
| SUB-02 | Safety Subsystem | Manual physical extraction | (`a5-prep-and-safety-rev7.md`) |
"""
            with open(os.path.join(reports_dir, "PHASE_1_EXECUTIVE_ENGINEERING_DELIVERABLES.md"), "w", encoding="utf-8") as f:
                f.write(content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo)
            table_findings = [f for f in findings if getattr(f, "rule_id", str(f)) == RULE_TABLE_UNANCHORED]
            self.assertEqual(table_findings, [])

    def test_diagram_with_realizes_and_coverage_passes(self):
        """Verify diagram with %% Realizes: and %% Coverage: metadata passes even if not encompassing all AST subsystems."""
        self._require_validator()
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            reports_dir = os.path.join(tmpdir, "docs", "reports")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(reports_dir, exist_ok=True)

            with open(os.path.join(schema_dir, "DEAP_MODEL.sysml"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_SYSML_MODEL)

            with open(os.path.join(reports_dir, "PHASE_1_EXECUTIVE_ENGINEERING_DELIVERABLES.md"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_DIAGRAM_WITH_REALIZES_AND_COVERAGE)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo)
            diag_findings = [f for f in findings if getattr(f, "rule_id", str(f)) == RULE_DIAGRAM_INCOMPLETE]
            self.assertEqual(diag_findings, [])

    def test_diagram_encompassing_all_ast_subsystems_passes(self):
        """Verify diagram encompassing all declared AST subsystems passes without explicit rationale."""
        self._require_validator()
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            reports_dir = os.path.join(tmpdir, "docs", "reports")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(reports_dir, exist_ok=True)

            with open(os.path.join(schema_dir, "DEAP_MODEL.sysml"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_SYSML_MODEL)

            with open(os.path.join(reports_dir, "PHASE_1_EXECUTIVE_ENGINEERING_DELIVERABLES.md"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_DIAGRAM_ENCOMPASSING_ALL_AST)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo)
            diag_findings = [f for f in findings if getattr(f, "rule_id", str(f)) == RULE_DIAGRAM_INCOMPLETE]
            self.assertEqual(diag_findings, [])

    def test_diagram_truncated_missing_subsystems_fails(self):
        """Verify failure when architecture diagram omits declared AST subsystems without scoping rationale."""
        self._require_validator()
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            reports_dir = os.path.join(tmpdir, "docs", "reports")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(reports_dir, exist_ok=True)

            with open(os.path.join(schema_dir, "DEAP_MODEL.sysml"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_SYSML_MODEL)

            with open(os.path.join(reports_dir, "PHASE_1_EXECUTIVE_ENGINEERING_DELIVERABLES.md"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_DIAGRAM_TRUNCATED_MISSING_SUBSYSTEMS)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo)
            diag_findings = [f for f in findings if getattr(f, "rule_id", str(f)) == RULE_DIAGRAM_INCOMPLETE]
            self.assertTrue(len(diag_findings) >= 1)
            self.assertTrue(any(RULE_DIAGRAM_INCOMPLETE in str(f) or "missing" in str(f).lower() or "incomplete" in str(f).lower() for f in diag_findings))

    def test_diagram_truncated_with_scoping_rationale_passes(self):
        """Verify truncated architecture diagram passes when accompanied by an explicit scoping rationale."""
        self._require_validator()
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            reports_dir = os.path.join(tmpdir, "docs", "reports")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(reports_dir, exist_ok=True)

            with open(os.path.join(schema_dir, "DEAP_MODEL.sysml"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_SYSML_MODEL)

            with open(os.path.join(reports_dir, "PHASE_1_EXECUTIVE_ENGINEERING_DELIVERABLES.md"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_DIAGRAM_TRUNCATED_WITH_SCOPING_RATIONALE)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo)
            diag_findings = [f for f in findings if getattr(f, "rule_id", str(f)) == RULE_DIAGRAM_INCOMPLETE]
            self.assertEqual(diag_findings, [])

    def test_clean_landing_zone_passes(self):
        """Verify clean landing zones (.gitkeep only) produce zero findings."""
        self._require_validator()
        with tempfile.TemporaryDirectory() as tmpdir:
            os.makedirs(os.path.join(tmpdir, "schema"), exist_ok=True)
            os.makedirs(os.path.join(tmpdir, "docs", "reports"), exist_ok=True)
            os.makedirs(os.path.join(tmpdir, "docs", "management"), exist_ok=True)
            with open(os.path.join(tmpdir, "schema", ".gitkeep"), "w") as f:
                f.write("")
            with open(os.path.join(tmpdir, "docs", "reports", ".gitkeep"), "w") as f:
                f.write("")
            with open(os.path.join(tmpdir, "docs", "management", ".gitkeep"), "w") as f:
                f.write("")

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo)
            self.assertEqual(findings, [])

    def test_check27_baseline_verification_integration(self):
        """Verify Check 27 integration in verify_downstream_baseline.py."""
        if check_executive_deliverable_traceability is None:
            self.fail("check_executive_deliverable_traceability is not implemented yet (RED phase)")

        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            reports_dir = os.path.join(tmpdir, "docs", "reports")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(reports_dir, exist_ok=True)

            with open(os.path.join(schema_dir, "DEAP_MODEL.sysml"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_SYSML_MODEL)

            with open(os.path.join(reports_dir, "PHASE_1_EXECUTIVE_ENGINEERING_DELIVERABLES.md"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_VALID_TABLE_REPORT)
                f.write("\n\n")
                f.write(SAMPLE_DIAGRAM_ENCOMPASSING_ALL_AST)

            # Clean/valid repository should pass Check 27 without exception
            check_executive_deliverable_traceability(tmpdir)
            if check_executive_deliverable_traceability_gate is not None:
                check_executive_deliverable_traceability_gate(tmpdir)

            # Violating repository should exit with error (code 1)
            with open(os.path.join(reports_dir, "PHASE_1_EXECUTIVE_ENGINEERING_DELIVERABLES.md"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_UNANCHORED_TABLE_REPORT)

            with self.assertRaises(SystemExit):
                check_executive_deliverable_traceability(tmpdir)


if __name__ == "__main__":
    unittest.main()

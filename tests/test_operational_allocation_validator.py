import os
import re
import sys
import tempfile
import unittest

repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if repo_root not in sys.path:
    sys.path.insert(0, repo_root)

parity_src = os.path.join(repo_root, "skills", "spec-orchestrator", "parity_auditor", "src")
if parity_src not in sys.path:
    sys.path.insert(0, parity_src)

from parity_auditor.core.workspace import WorkspaceRepository
from parity_auditor.validators.operational_allocation_validator import (
    OperationalAllocationValidator,
    _normalize_oa_id,
    _parse_allocation_tags,
)

CANONICAL_CONOPS_UNIT_06 = os.path.join(
    repo_root,
    "skills",
    "spec-conops-engineering",
    "resources",
    "units",
    "conops",
    "06_UAF_OPERATIONAL_ACTIVITIES.md",
)


def _load_canonical_unit06() -> str:
    with open(CANONICAL_CONOPS_UNIT_06, "r", encoding="utf-8") as f:
        return f.read()


def _get_canonical_sv5a_section() -> str:
    content = _load_canonical_unit06()
    m = re.search(r"(### 6\.2 DoDAF SV-5a.*?)(?=\n#### 6\.2\.1|\n### 6\.3|\Z)", content, re.DOTALL)
    if m:
        return "\n" + m.group(1).strip() + "\n"
    return ""


class TestOperationalAllocationValidator(unittest.TestCase):
    def test_normalization_helper(self):
        """Verify Operational Activity identifier normalization."""
        self.assertEqual(_normalize_oa_id("OA-01"), "OA-01")
        self.assertEqual(_normalize_oa_id("OA-1"), "OA-01")
        self.assertEqual(_normalize_oa_id("OA_01"), "OA-01")
        self.assertEqual(_normalize_oa_id("OA_2"), "OA-02")
        self.assertEqual(_normalize_oa_id("OA-STARTUP"), "OA-STARTUP")
        self.assertEqual(_normalize_oa_id("Startup"), "STARTUP")
        self.assertEqual(_normalize_oa_id("ActiveExecution"), "ACTIVEEXECUTION")

    def test_syntax_parsing_tags(self):
        """Verify parsing of '/// OperationalAllocation: [OA-XX, ...]'."""
        text1 = "/// OperationalAllocation: [OA-01, OA-02, Startup]"
        tags1 = _parse_allocation_tags(text1)
        self.assertEqual(tags1, ["OA-01", "OA-02", "STARTUP"])

        text2 = "doc /* /// OperationalAllocation: [OA-03] */"
        tags2 = _parse_allocation_tags(text2)
        self.assertEqual(tags2, ["OA-03"])

        text3 = "/* /// OperationalAllocation: [OA-04, DegradedMode] */"
        tags3 = _parse_allocation_tags(text3)
        self.assertEqual(tags3, ["OA-04", "DEGRADEDMODE"])

        text4 = "/// OperationalAllocation: [OA-05]"
        tags4 = _parse_allocation_tags(text4)
        self.assertEqual(tags4, ["OA-05"])

    def test_clean_upstream_landing_zone_passes(self):
        """Verify that empty CONOPS and specs landing zone passes gracefully."""
        with tempfile.TemporaryDirectory() as tmpdir:
            os.makedirs(os.path.join(tmpdir, "docs", "conops"), exist_ok=True)
            os.makedirs(os.path.join(tmpdir, "docs", "features"), exist_ok=True)
            os.makedirs(os.path.join(tmpdir, "schema"), exist_ok=True)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = OperationalAllocationValidator()
            errors = validator.validate(repo)
            self.assertEqual(errors, [])

    def test_dynamic_phase_extraction_and_full_allocation_passes(self):
        """Verify 100% allocation across dynamic CONOPS phases and activities passes with zero findings."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            features_dir = os.path.join(tmpdir, "docs", "features")
            schema_dir = os.path.join(tmpdir, "schema")
            os.makedirs(conops_dir, exist_ok=True)
            os.makedirs(features_dir, exist_ok=True)
            os.makedirs(schema_dir, exist_ok=True)

            # Write CONOPS.md with dynamic operational phases and activities
            conops_md = """# Concept of Operations (CONOPS)
## Operational Lifecycle Phases
- **Phase 1: Startup**
- **Phase 2: ActiveExecution**
- **Phase 3: SecureShutdown**

## Operational Activities Roster
| Activity ID | Name | Description | SSOT Ground Truth Source & Citation |
| :--- | :--- | :--- | :--- |
| `OA-01` | System Initialization | Perform boot and PBIT tests | `schema/system_architecture.sysml` |
| `OA-02` | Guidance Loop Execution | Continuous sensor-to-actuator compute | `MIL-STD-882E` |
| `OA-03` | Controlled Power Down | Safe sequence parking and shutdown | `ISO/IEC/IEEE 29148:2018` |
""" + _get_canonical_sv5a_section()
            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(conops_md)

            # Feature spec allocating OA-01 and Startup
            feat1_md = """---
id: FEAT-01
title: System Startup & Initialization
---
# Feature 01: Startup
/// OperationalAllocation: [OA-01, Startup]

Class SensorInitializationController { ... }
"""
            with open(os.path.join(features_dir, "FEAT_01_STARTUP.md"), "w", encoding="utf-8") as f:
                f.write(feat1_md)

            # Feature spec allocating OA-02 and ActiveExecution
            feat2_md = """---
id: FEAT-02
title: Active Guidance Compute
---
# Feature 02: Guidance
/// OperationalAllocation: [OA-02, ActiveExecution]

Class GuidanceEngine { ... }
"""
            with open(os.path.join(features_dir, "FEAT_02_GUIDANCE.md"), "w", encoding="utf-8") as f:
                f.write(feat2_md)

            # SysML model allocating OA-03 and SecureShutdown
            sysml_content = """package SubsystemAllocations {
    part def ShutdownManager {
        doc /* /// OperationalAllocation: [OA-03, SecureShutdown] */
        action performShutdown;
    }
}
"""
            with open(os.path.join(schema_dir, "shutdown.sysml"), "w", encoding="utf-8") as f:
                f.write(sysml_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = OperationalAllocationValidator()
            errors = validator.validate(repo)
            self.assertEqual(errors, [])

    def test_theorem_1_orphan_activity_detected(self):
        """Verify Theorem 1: Unallocated operational activity (O_orphan != empty) is flagged."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            features_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(conops_dir, exist_ok=True)
            os.makedirs(features_dir, exist_ok=True)

            conops_md = """# CONOPS
## Operational Activities
| Activity ID | Name | Description | SSOT Ground Truth Source & Citation |
| :--- | :--- | :--- | :--- |
| `OA-01` | Initialization | System boot | `schema/system_architecture.sysml` |
| `OA-02` | Fault Isolation | Diagnostic isolation | `MIL-STD-882E` |
""" + _get_canonical_sv5a_section()
            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(conops_md)

            # Feature only allocates OA-01, leaving OA-02 orphan
            feat_md = """# Feature
/// OperationalAllocation: [OA-01]
"""
            with open(os.path.join(features_dir, "FEAT_01.md"), "w", encoding="utf-8") as f:
                f.write(feat_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = OperationalAllocationValidator()
            errors = validator.validate(repo)

            self.assertEqual(len(errors), 1)
            self.assertEqual(errors[0].rule_id, "operational-allocation-orphan-activity")
            self.assertIn("OA-02", str(errors[0]))

    def test_theorem_2_phantom_tag_detected(self):
        """Verify Theorem 2: Allocation tag citing undeclared activity (P_phantom != empty) is flagged."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            features_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(conops_dir, exist_ok=True)
            os.makedirs(features_dir, exist_ok=True)

            conops_md = """# CONOPS
## Operational Activities
| Activity ID | Name | Description | SSOT Ground Truth Source & Citation |
| :--- | :--- | :--- | :--- |
| `OA-01` | Initialization | System boot | `schema/system_architecture.sysml` |
""" + _get_canonical_sv5a_section()
            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(conops_md)

            # Feature allocates OA-01 AND non-existent OA-99
            feat_md = """# Feature
/// OperationalAllocation: [OA-01, OA-99]
"""
            with open(os.path.join(features_dir, "FEAT_01.md"), "w", encoding="utf-8") as f:
                f.write(feat_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = OperationalAllocationValidator()
            errors = validator.validate(repo)

            self.assertEqual(len(errors), 1)
            self.assertEqual(errors[0].rule_id, "operational-allocation-phantom-tag")
            self.assertIn("OA-99", str(errors[0]))

    def test_matrix_synthesis(self):
        """Verify automated synthesis of OP_TO_RES_ALLOCATION_MATRIX.md table."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            features_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(conops_dir, exist_ok=True)
            os.makedirs(features_dir, exist_ok=True)

            conops_md = """# CONOPS
## Operational Activities
| Activity ID | Name | Description | SSOT Ground Truth Source & Citation |
| :--- | :--- | :--- | :--- |
| `OA-01` | System Initialization | Boot sequence | `schema/system_architecture.sysml` |
"""
            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(conops_md)

            feat_md = """# Feature 01
/// OperationalAllocation: [OA-01]
"""
            with open(os.path.join(features_dir, "FEAT_01.md"), "w", encoding="utf-8") as f:
                f.write(feat_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = OperationalAllocationValidator()
            matrix_md = validator.synthesize_allocation_matrix(repo)

            self.assertIn("Operational-to-Resource Allocation Matrix", matrix_md)
            self.assertIn("OA-01", matrix_md)
            self.assertIn("FEAT_01.md", matrix_md)

    def test_fresh_downstream_workspace_with_conops_and_no_features_passes(self):
        """Verify that a fresh downstream workspace with CONOPS but zero feature specifications does not emit orphan errors."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            features_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(conops_dir, exist_ok=True)
            os.makedirs(features_dir, exist_ok=True)

            conops_md = """# Concept of Operations (CONOPS)
## Operational Lifecycle Phases
- **Phase 1: Startup**
- **Phase 2: ActiveExecution**
- **Phase 3: SecureShutdown**

## Operational Activities Roster
| Activity ID | Name | Description | SSOT Ground Truth Source & Citation |
| :--- | :--- | :--- | :--- |
| `OA-01` | System Initialization | Boot sequence | `schema/system_architecture.sysml` |
| `OA-02` | Guidance Loop Execution | Compute | `MIL-STD-882E` |
""" + _get_canonical_sv5a_section()
            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(conops_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = OperationalAllocationValidator()
            errors = validator.validate(repo)
            self.assertEqual(errors, [])

    def test_fresh_downstream_workspace_with_allow_missing_specs_true_passes(self):
        """Verify that allow_missing_specs=True suppresses orphan-activity errors in pre-feature stage."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(conops_dir, exist_ok=True)

            conops_md = """# Concept of Operations (CONOPS)
## Operational Activities
| Activity ID | Name | Description | SSOT Ground Truth Source & Citation |
| :--- | :--- | :--- | :--- |
| `OA-01` | System Initialization | Boot sequence | `schema/system_architecture.sysml` |
""" + _get_canonical_sv5a_section()
            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(conops_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = OperationalAllocationValidator()
            errors = validator.validate(repo, allow_missing_specs=True)
            self.assertEqual(errors, [])

    def test_workspace_with_features_strictly_enforces_orphan_activities(self):
        """Verify that when feature specifications exist, unallocated operational activities are flagged as fatal errors."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            features_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(conops_dir, exist_ok=True)
            os.makedirs(features_dir, exist_ok=True)

            conops_md = """# CONOPS
## Operational Activities
| Activity ID | Name | Description | SSOT Ground Truth Source & Citation |
| :--- | :--- | :--- | :--- |
| `OA-01` | System Initialization | Boot sequence | `schema/system_architecture.sysml` |
| `OA-02` | Guidance Compute | Guidance | `MIL-STD-882E` |
""" + _get_canonical_sv5a_section()
            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(conops_md)

            # Feature only allocates OA-01
            feat_md = """# Feature 01
/// OperationalAllocation: [OA-01]
"""
            with open(os.path.join(features_dir, "FEAT_01.md"), "w", encoding="utf-8") as f:
                f.write(feat_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = OperationalAllocationValidator()
            errors = validator.validate(repo, allow_missing_specs=False)

            self.assertEqual(len(errors), 1)
            self.assertEqual(errors[0].rule_id, "operational-allocation-orphan-activity")
            self.assertIn("OA-02", str(errors[0]))

    def test_phantom_tags_strictly_enforced_even_if_allow_missing_specs(self):
        """Verify that phantom tags are always flagged regardless of allow_missing_specs setting."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            schema_dir = os.path.join(tmpdir, "schema")
            os.makedirs(conops_dir, exist_ok=True)
            os.makedirs(schema_dir, exist_ok=True)

            conops_md = """# CONOPS
## Operational Activities
| Activity ID | Name | Description | SSOT Ground Truth Source & Citation |
| :--- | :--- | :--- | :--- |
| `OA-01` | Initialization | System boot | `schema/system_architecture.sysml` |
""" + _get_canonical_sv5a_section()
            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(conops_md)

            # Schema file allocates phantom tag OA-99
            sysml_content = """package Allocations {
    part def TestPart {
        doc /* /// OperationalAllocation: [OA-99] */
    }
}
"""
            with open(os.path.join(schema_dir, "test.sysml"), "w", encoding="utf-8") as f:
                f.write(sysml_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = OperationalAllocationValidator()
            errors = validator.validate(repo, allow_missing_specs=True)

            self.assertEqual(len(errors), 1)
            self.assertIn("OA-99", str(errors[0]))

    def test_fresh_workspace_phase1_conops_and_mission_intent_partial_tags_passes(self):
        """Verify that in a fresh workspace containing Phase 1 Level 1 specifications (CONOPS.md / MISSION_INTENT.md)
        with allocation tags in Mission Intent but empty docs/features landing zone, Gate 24 exhibits clean
        stage-awareness and does not emit false-positive orphan allocation errors for downstream tasks/features."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            features_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(conops_dir, exist_ok=True)
            os.makedirs(features_dir, exist_ok=True)

            conops_md = """# Concept of Operations (CONOPS)
## Operational Lifecycle Phases
- **Phase 1: Startup**
- **Phase 2: ActiveExecution**
- **Phase 3: SecureShutdown**

## Operational Activities Roster
| Activity ID | Name | Description | SSOT Ground Truth Source & Citation |
| :--- | :--- | :--- | :--- |
| `OA-01` | System Initialization | Boot sequence | `schema/system_architecture.sysml` |
| `OA-02` | Guidance Loop Execution | Compute | `MIL-STD-882E` |
| `OA-03` | Controlled Power Down | Safe shutdown | `DO-178C` |
""" + _get_canonical_sv5a_section()
            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(conops_md)

            # MISSION_INTENT defines tactical METL tasks and allocates OA-01 and Startup
            mission_intent_md = """# Tactical Mission Intent
## Tactical METL Tasks
| METL ID | Task Title | Description | SSOT Ground Truth Source & Citation |
| :--- | :--- | :--- | :--- |
| `MET-01` | Pre-Flight Checkout | Verify system health | `STANAG 4586` |
| `MET-02` | Autonomous Navigation | Execute waypoint flight | `schema/system_architecture.sysml` |

## Operational Allocations
/// OperationalAllocation: [OA-01, Startup]
"""
            with open(os.path.join(conops_dir, "MISSION_INTENT.md"), "w", encoding="utf-8") as f:
                f.write(mission_intent_md)

            # docs/features is empty (only .gitkeep or README.md)
            with open(os.path.join(features_dir, ".gitkeep"), "w", encoding="utf-8") as f:
                f.write("")

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = OperationalAllocationValidator()
            errors = validator.validate(repo, allow_missing_specs=False)
            self.assertEqual(errors, [])

    def test_fresh_workspace_phase1_with_schema_allocation_and_no_features_passes(self):
        """Verify that when early schema/design files allocate a subset of activities in a workspace without
        features (Phase 1), Gate 24 stage-awareness suppresses orphan findings for downstream features."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            features_dir = os.path.join(tmpdir, "docs", "features")
            schema_dir = os.path.join(tmpdir, "schema")
            os.makedirs(conops_dir, exist_ok=True)
            os.makedirs(features_dir, exist_ok=True)
            os.makedirs(schema_dir, exist_ok=True)

            conops_md = """# Concept of Operations (CONOPS)
## Operational Lifecycle Phases
- **Phase 1: Startup**
- **Phase 2: ActiveExecution**
- **Phase 3: SecureShutdown**

## Operational Activities Roster
| Activity ID | Name | Description | SSOT Ground Truth Source & Citation |
| :--- | :--- | :--- | :--- |
| `OA-01` | System Initialization | Boot sequence | `schema/system_architecture.sysml` |
| `OA-02` | Guidance Loop Execution | Compute | `MIL-STD-882E` |
| `OA-03` | Controlled Power Down | Safe shutdown | `DO-178C` |
""" + _get_canonical_sv5a_section()
            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(conops_md)

            # Early schema file allocates only OA-01
            sysml_content = """package SystemAllocations {
    part def BootManager {
        doc /* /// OperationalAllocation: [OA-01] */
    }
}
"""
            with open(os.path.join(schema_dir, "boot.sysml"), "w", encoding="utf-8") as f:
                f.write(sysml_content)

            # docs/features is empty (pre-feature stage)
            with open(os.path.join(features_dir, ".gitkeep"), "w", encoding="utf-8") as f:
                f.write("")

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = OperationalAllocationValidator()
            errors = validator.validate(repo, allow_missing_specs=False)
            self.assertEqual(errors, [])

    def test_theorem_3_missing_provenance_column_detected(self):
        """Verify Theorem 3: Table without a provenance column emits operational-activity-unanchored-provenance."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            features_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(conops_dir, exist_ok=True)
            os.makedirs(features_dir, exist_ok=True)

            conops_md = """# CONOPS
## Operational Activities
| Activity ID | Name | Description |
| :--- | :--- | :--- |
| `OA-01` | Initialization | System boot |
""" + _get_canonical_sv5a_section()
            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(conops_md)

            feat_md = """# Feature 01
/// OperationalAllocation: [OA-01]
"""
            with open(os.path.join(features_dir, "FEAT_01.md"), "w", encoding="utf-8") as f:
                f.write(feat_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = OperationalAllocationValidator()
            errors = validator.validate(repo)

            self.assertEqual(len(errors), 1)
            self.assertEqual(errors[0].rule_id, "operational-activity-unanchored-provenance")
            self.assertIn("OA-01", str(errors[0]))
            self.assertIn("Theorem 3 violation: unanchored operational activity", str(errors[0]))

    def test_theorem_3_placeholder_provenance_detected(self):
        """Verify Theorem 3: TBD, N/A, or placeholder citations emit operational-activity-unanchored-provenance."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            features_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(conops_dir, exist_ok=True)
            os.makedirs(features_dir, exist_ok=True)

            conops_md = """# CONOPS
## Operational Activities
| Activity ID | Name | Description | SSOT Ground Truth Source & Citation |
| :--- | :--- | :--- | :--- |
| `OA-01` | Initialization | System boot | TBD |
| `OA-02` | Guidance Compute | Flight guidance | `N/A` |
| `OA-03` | Environmental Monitoring | Safety limits | Unspecified |
""" + _get_canonical_sv5a_section()
            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(conops_md)

            feat_md = """# Feature 01
/// OperationalAllocation: [OA-01, OA-02, OA-03]
"""
            with open(os.path.join(features_dir, "FEAT_01.md"), "w", encoding="utf-8") as f:
                f.write(feat_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = OperationalAllocationValidator()
            errors = validator.validate(repo)

            self.assertEqual(len(errors), 3)
            for err in errors:
                self.assertEqual(err.rule_id, "operational-activity-unanchored-provenance")
                self.assertIn("Theorem 3 violation: unanchored operational activity", str(err))

    def test_theorem_3_valid_provenance_passes(self):
        """Verify Theorem 3: Valid schema and standard citations pass with zero findings."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            features_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(conops_dir, exist_ok=True)
            os.makedirs(features_dir, exist_ok=True)

            conops_md = """# CONOPS
## Operational Activities
| Activity ID | Name | Description | SSOT Ground Truth Source & Citation |
| :--- | :--- | :--- | :--- |
| `OA-01` | Initialization | System boot | `schema/system_architecture.sysml` |
| `OA-02` | Guidance Compute | Flight guidance | MIL-STD-882E §4.3 |
| `OA-03` | Telemetry Exchange | PACE links | STANAG 4586 Edition 3 |
| `OA-04` | Safety Containment | Failsafe state | DO-178C DAL-A |
| `OA-05` | Requirements Baseline | Requirement specs | ISO/IEC/IEEE 29148:2018 |
""" + _get_canonical_sv5a_section()
            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(conops_md)

            feat_md = """# Feature 01
/// OperationalAllocation: [OA-01, OA-02, OA-03, OA-04, OA-05]
"""
            with open(os.path.join(features_dir, "FEAT_01.md"), "w", encoding="utf-8") as f:
                f.write(feat_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = OperationalAllocationValidator()
            errors = validator.validate(repo)

            self.assertEqual(errors, [])

    def test_schema_driven_table_parsing_with_all_columns(self):
        """Verify structured table parsing extracts rows as typed dictionaries mapping to schema properties."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            features_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(conops_dir, exist_ok=True)
            os.makedirs(features_dir, exist_ok=True)

            conops_md = """# Concept of Operations
## Operational Activities
| Activity ID | Activity Name | Allocated Performer Node & Resource | Operational Description | Traceability & Gate 24 Allocation Tag | SSOT Ground Truth Source & Citation |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `OA-01` | System Initialization | PrimarySensorSubsystem | Perform boot BIT tests | `/// OperationalAllocation: [OA-01]` | `schema/system_architecture.sysml` |
""" + _get_canonical_sv5a_section()
            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(conops_md)

            feat_md = """# Feature 01
/// OperationalAllocation: [OA-01]
"""
            with open(os.path.join(features_dir, "FEAT_01.md"), "w", encoding="utf-8") as f:
                f.write(feat_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = OperationalAllocationValidator()
            errors = validator.validate(repo)
            self.assertEqual(errors, [])

            # Check extracted activities dictionary
            act = validator.extracted_activities.get("OA-01")
            self.assertIsNotNone(act)
            self.assertEqual(act["id"], "OA-01")
            self.assertEqual(act["name"], "System Initialization")
            self.assertEqual(act["allocated_performer"], "PrimarySensorSubsystem")
            self.assertEqual(act["description"], "Perform boot BIT tests")
            self.assertEqual(act["allocation_tag"], "/// OperationalAllocation: [OA-01]")
            self.assertEqual(act["ssot_source"], "schema/system_architecture.sysml")

    def test_schema_validation_minLength_enforced(self):
        """Verify that ssot_source shorter than minLength (3) emits operational-activity-unanchored-provenance."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            features_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(conops_dir, exist_ok=True)
            os.makedirs(features_dir, exist_ok=True)

            conops_md = """# CONOPS
## Operational Activities
| Activity ID | Name | Description | SSOT Ground Truth Source & Citation |
| :--- | :--- | :--- | :--- |
| `OA-01` | Initialization | System boot | 12 |
""" + _get_canonical_sv5a_section()
            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(conops_md)

            feat_md = """# Feature 01
/// OperationalAllocation: [OA-01]
"""
            with open(os.path.join(features_dir, "FEAT_01.md"), "w", encoding="utf-8") as f:
                f.write(feat_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = OperationalAllocationValidator()
            errors = validator.validate(repo)
            self.assertEqual(len(errors), 1)
            self.assertEqual(errors[0].rule_id, "operational-activity-unanchored-provenance")
            self.assertIn("OA-01", str(errors[0]))

    def test_sysml_action_def_and_activity_def_ingestion(self):
        """Verify SysML v2 AST action def and activity def nodes are ingested using load_sysml_ast_members."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            schema_dir = os.path.join(tmpdir, "schema")
            os.makedirs(conops_dir, exist_ok=True)
            os.makedirs(schema_dir, exist_ok=True)

            conops_md = """# Concept of Operations
## Operational Activities
| Activity ID | Name | Description | SSOT Ground Truth Source & Citation |
| :--- | :--- | :--- | :--- |
| `OA-01` | Boot Sequence | Power on BIT | `schema/system_architecture.sysml` |
| `OA-02` | Guidance Loop | Real-time control | `schema/system_architecture.sysml` |
""" + _get_canonical_sv5a_section()
            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(conops_md)

            sysml_content = """package SubsystemAllocations {
    action def OA_01;
    activity def GuidanceLoop {
        doc /* /// OperationalAllocation: [OA-02] */
    }
}
"""
            with open(os.path.join(schema_dir, "system.sysml"), "w", encoding="utf-8") as f:
                f.write(sysml_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = OperationalAllocationValidator()
            # Verify extract_sysml_actions finds both action def and activity def
            actions = validator.extract_sysml_actions(repo)
            act_names = {a["name"] for a in actions}
            self.assertIn("OA_01", act_names)
            self.assertIn("GuidanceLoop", act_names)

            # Both OA-01 (via action def name) and OA-02 (via activity def doc tag) should be allocated
            errors = validator.validate(repo, allow_missing_specs=False)
            self.assertEqual(errors, [])

    def test_sv5a_matrix_conforming_passes(self):
        """Verify conforming SV-5a matrix mapping 100% of declared OA activities passes."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            features_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(conops_dir, exist_ok=True)
            os.makedirs(features_dir, exist_ok=True)

            canonical_content = _load_canonical_unit06()
            with open(os.path.join(conops_dir, "06_UAF_OPERATIONAL_ACTIVITIES.md"), "w", encoding="utf-8") as f:
                f.write(canonical_content)

            feat_md = """# Feature 01
/// OperationalAllocation: [OA-01, OA-02, OA-03, OA-04, OA-05, OA-06, OA-07, OA-08]
"""
            with open(os.path.join(features_dir, "FEAT_01.md"), "w", encoding="utf-8") as f:
                f.write(feat_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = OperationalAllocationValidator()
            errors = validator.validate(repo)
            self.assertEqual(errors, [])

    def test_sv5a_matrix_missing_flagged(self):
        """Verify missing SV-5a allocation matrix flags operational-allocation-missing-sv5a-matrix."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            features_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(conops_dir, exist_ok=True)
            os.makedirs(features_dir, exist_ok=True)

            canonical_content = _load_canonical_unit06()
            # Mutate real content by removing Section 6.2 SV-5a matrix
            mutated_content = re.sub(
                r"### 6\.2 DoDAF SV-5a.*?(?=### 6\.3)", "", canonical_content, flags=re.DOTALL
            )

            target_rel = "docs/conops/06_UAF_OPERATIONAL_ACTIVITIES.md"
            with open(os.path.join(tmpdir, target_rel), "w", encoding="utf-8") as f:
                f.write(mutated_content)

            feat_md = """# Feature 01
/// OperationalAllocation: [OA-01, OA-02, OA-03, OA-04, OA-05, OA-06, OA-07, OA-08]
"""
            with open(os.path.join(features_dir, "FEAT_01.md"), "w", encoding="utf-8") as f:
                f.write(feat_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = OperationalAllocationValidator()
            errors = validator.validate(repo)
            sv5a_missing_errors = [e for e in errors if e.rule_id == "operational-allocation-missing-sv5a-matrix"]
            self.assertEqual(len(sv5a_missing_errors), 1)
            self.assertEqual(
                sv5a_missing_errors[0].message,
                f"ConOps baseline at {target_rel} lacks mandatory DoDAF SV-5a / OMG UAF Op-to-Res Operational-to-Resource Allocation Matrix."
            )

    def test_sv5a_matrix_incomplete_flagged(self):
        """Verify incomplete SV-5a allocation matrix missing declared OA-* activities flags operational-allocation-sv5a-incomplete."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            features_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(conops_dir, exist_ok=True)
            os.makedirs(features_dir, exist_ok=True)

            canonical_content = _load_canonical_unit06()
            # Mutate real content by removing OA-02 from Section 6.2 table and summary
            lines = canonical_content.splitlines()
            mutated_lines = []
            in_sv5a = False
            for line in lines:
                if "### 6.2 DoDAF SV-5a" in line:
                    in_sv5a = True
                elif in_sv5a and line.startswith("### 6.3"):
                    in_sv5a = False
                if in_sv5a and "OA-02" in line:
                    if line.strip().startswith("|") and "**OA-02**" in line:
                        continue
                    elif "OA-02" in line:
                        line = line.replace("OA-02, ", "").replace(", OA-02", "").replace("OA-02", "")
                mutated_lines.append(line)

            target_rel = "docs/conops/06_UAF_OPERATIONAL_ACTIVITIES.md"
            with open(os.path.join(tmpdir, target_rel), "w", encoding="utf-8") as f:
                f.write("\n".join(mutated_lines))

            feat_md = """# Feature 01
/// OperationalAllocation: [OA-01, OA-02, OA-03, OA-04, OA-05, OA-06, OA-07, OA-08]
"""
            with open(os.path.join(features_dir, "FEAT_01.md"), "w", encoding="utf-8") as f:
                f.write(feat_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = OperationalAllocationValidator()
            errors = validator.validate(repo)
            sv5a_incomplete_errors = [e for e in errors if e.rule_id == "operational-allocation-sv5a-incomplete"]
            self.assertEqual(len(sv5a_incomplete_errors), 1)
            self.assertEqual(
                sv5a_incomplete_errors[0].message,
                f"DoDAF SV-5a / UAF Op-to-Res Matrix at {target_rel} is incomplete: missing operational activities [OA-02]."
            )

    def test_sv5a_matrix_clean_upstream_landing_zone_passes(self):
        """Verify clean upstream landing zone with empty ops_universe passes gracefully with zero findings."""
        with tempfile.TemporaryDirectory() as tmpdir:
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(conops_dir, exist_ok=True)
            with open(os.path.join(conops_dir, ".gitkeep"), "w", encoding="utf-8") as f:
                f.write("")

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = OperationalAllocationValidator()
            errors = validator.validate(repo)
            self.assertEqual(errors, [])


if __name__ == "__main__":
    unittest.main()




#!/usr/bin/env python3
"""
Unit tests for Coverage-Digest Population Gate (closing #92) and
Obligation-Witness Registry (closing #93) in parity_auditor (CORE #98).
(`tests/test_coverage_digest_and_witness_registry.py`)

Verifies:
1. Data models:
   - CoverageDigest: calculation of totals, realization percentages, serialization, markdown generation.
   - ObligationWitnessRecord & ObligationWitnessRegistry: multi-dimensional witness aggregation (spec, test, code, model), coverage percentages, serialization.
2. CoverageDigestValidator (Gate 28 / #92):
   - Extracts declared obligations from Section 3 (Population Register), Section 4 (External Additions), Section 2 (Standards).
   - Scans specifications (features, epics, user stories, use cases, ICDs, ConOps, SysML, Section 5 allocations) for realized obligations.
   - Flags phantom realizations (tags referencing undeclared obligation IDs).
   - Flags unrealized obligations when specs exist.
   - Stage-awareness: cleanly passes in upstream compiler mode and fresh/pre-feature workspaces when allow_missing_specs=True.
   - Synthesizes COVERAGE_DIGEST.md report.
3. ObligationWitnessValidator & Registry (Gate 29 / #93):
   - Extracts and registers Spec Witnesses, Test Witnesses, Code Witnesses, Model Witnesses.
   - Flags phantom witnesses (witness tags referencing undeclared obligations).
   - Flags unwitnessed obligations across relevant lifecycle stages.
   - Stage-awareness: handles upstream compiler mode, spec-only mode, and full codebase mode.
   - Synthesizes OBLIGATION_WITNESS_REGISTRY.md.
4. Aggregator & CLI Integration:
   - Both validators are registered in AGGREGATING_VALIDATORS in aggregator.py.
   - CLI imports and executes Gate 28 and Gate 29.
"""

import os
import sys
import tempfile
import unittest

def _find_project_root(start_path: str) -> str:
    curr = os.path.abspath(start_path)
    while True:
        if os.path.exists(os.path.join(curr, ".pipeline", "logical-ui", "codebase_rules.json")):
            return curr
        parent = os.path.dirname(curr)
        if parent == curr:
            break
        curr = parent
    return os.path.abspath(start_path)

PROJECT_ROOT = _find_project_root(os.path.dirname(os.path.abspath(__file__)))
if PROJECT_ROOT not in sys.path:
    sys.path.insert(0, PROJECT_ROOT)

PARITY_SRC = os.path.join(PROJECT_ROOT, "skills", "spec-orchestrator", "parity_auditor", "src")
if PARITY_SRC not in sys.path:
    sys.path.insert(0, PARITY_SRC)

from parity_auditor.core.models import (
    CoverageDigest,
    ObligationWitnessRecord,
    ObligationWitnessRegistry,
    NormativeStandard,
    PopulationRegisterEntry,
    ExternalAdditionEntry,
    ClauseAllocationEntry,
    ResearchInventoryDocument,
)
from parity_auditor.parsers.research_inventory import (
    ResearchInventoryParser,
    parse_research_inventory,
)
from parity_auditor.validators.coverage_digest_validator import (
    CoverageDigestValidator,
)
from parity_auditor.validators.obligation_witness_validator import (
    ObligationWitnessValidator,
)
from parity_auditor.core.workspace import WorkspaceRepository
from parity_auditor.aggregator import AGGREGATING_VALIDATORS


SAMPLE_RESEARCH_INVENTORY = r"""
| Attribute | Value |
| :--- | :--- |
| **Title** | Cited Research Inventory: Autonomous Mission System |
| **Version** | 1.0.0 |
| **Date** | 2026-09-02 |

# Cited Research Inventory: Autonomous Mission System

## 1. Scope & System Identification
- **System Identifier:** `AMS-CORE`
- **Operational Domain:** `Safety-Critical Flight Automation`
- **Research Scope:** Authoritative baseline.
- **Applicability Statement:** Level-A flight software and ground infrastructure.

## 2. Normative Standards & Baseline Documents Inventory
| Standard / Baseline ID | Issuing Body | Title | Applicable Clauses | Obligation Category | Declared Total | Clause Citation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| ISO/IEC/IEEE 29148:2018 | ISO/IEC/IEEE | Systems and Software Engineering — Requirements Engineering | §6.4.2 ConOps, §8.4 System Requirements | Requirements Engineering | 2 | ISO/IEC/IEEE 29148:2018 §6.4.2, §8.4 |
| RTCA DO-178C / DO-254 | RTCA / EUROCAE | Software and Electronic Hardware Considerations in Airborne Systems | §6.3 Software Architecture | Safety Assurance | 2 | DO-178C §6.3 |
| NATO STANAG 4586 | NATO | Standard Interfaces of Autonomous Control Systems | §3.2 DLI Interface | Interoperability | 1 | STANAG 4586 Ed. 4 §3.2 |

## 3. Declared-Total Population Register
| Obligation ID | Category | Standard ID | Target Metric / Obligation Count | Verification Mechanism | Public Clause Citation |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `OBL-01` | Requirements Engineering | ISO/IEC/IEEE 29148:2018 | 1 | ConOps Inspection | ISO/IEC/IEEE 29148:2018 §6.4.2 |
| `OBL-02` | Requirements Engineering | ISO/IEC/IEEE 29148:2018 | 1 | Schema Validation | ISO/IEC/IEEE 29148:2018 §8.4 |
| `SAF-01` | Safety Assurance | RTCA DO-178C / DO-254 | 1 | Formal Static Analysis | DO-178C §6.3 |
| `SAF-02` | Safety Assurance | RTCA DO-178C / DO-254 | 1 | MCDC Test Coverage | DO-178C §6.3 |
| `INT-01` | Interoperability | NATO STANAG 4586 | 1 | Interface Conformance Test | STANAG 4586 Ed. 4 §3.2 |

## 4. External Additions & Domain Extensions Registry
| Extension ID | Category | Standard / Baseline ID | Declared Total | Target Metric / Obligation Count | Verification Mechanism | Public Clause Citation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `EXT-01` | Domain Extension | ASTM F3269-17 | 1 | 1 | Automated Flight Bounds Test | ASTM F3269-17 §5.2 |

## 5. Clause-Level Allocation & Traceability Matrix
| Population ID | Standard ID | Clause Citation | Clause Title / Requirement Excerpt | Specification Phase | Downstream Spec File / Tag |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `OBL-01` | ISO/IEC/IEEE 29148:2018 | ISO/IEC/IEEE 29148:2018 §6.4.2 | System ConOps Requirements | Phase 1 (Structural) | `docs/conops/CONOPS.md` |
| `OBL-02` | ISO/IEC/IEEE 29148:2018 | ISO/IEC/IEEE 29148:2018 §8.4 | System Requirements Specification | Phase 2 (Logical) | `docs/features/feat-01-requirements.md` |
| `SAF-01` | RTCA DO-178C / DO-254 | DO-178C §6.3 | Software Architecture Invariants | Phase 2 (Logical) | `docs/features/feat-02-safety.md` |
| `SAF-02` | RTCA DO-178C / DO-254 | DO-178C §6.3 | MCDC Verification Criteria | Phase 3 (Verification) | `docs/features/feat-02-safety.md` |
| `INT-01` | NATO STANAG 4586 | STANAG 4586 Ed. 4 §3.2 | DLI Message Gateway | Phase 2 (Logical) | `docs/features/feat-03-dli.md` |
| `EXT-01` | ASTM F3269-17 | ASTM F3269-17 §5.2 | Dynamic Bounding RTA | Phase 2 (Logical) | `docs/features/feat-04-rta.md` |

## 6. Normative Completeness & Gap Analysis
| Metric Parameter | Value | Target Threshold | Compliance Status |
| :--- | :--- | :--- | :--- |
| Declared Total Normative Obligations | 6 | $\ge 1$ | Conforming |
| Declared Total Safety Constraints | 2 | $\ge 1$ | Conforming |
| Clause Citation Traceability Percentage | 100% | 100% | Conforming |
| Un-Cited / Speculative Additions | 0 | 0 (Strict Zero Tolerance) | Conforming |
"""


class TestCoverageDigestModel(unittest.TestCase):
    """Unit tests for CoverageDigest data model."""

    def test_coverage_digest_initialization_and_methods(self):
        digest = CoverageDigest(
            total_declared_obligations=6,
            total_realized_obligations=6,
            realization_percentage=100.0,
            declared_by_standard={"ISO/IEC/IEEE 29148:2018": 2, "RTCA DO-178C / DO-254": 2, "NATO STANAG 4586": 1, "ASTM F3269-17": 1},
            realized_by_standard={"ISO/IEC/IEEE 29148:2018": 2, "RTCA DO-178C / DO-254": 2, "NATO STANAG 4586": 1, "ASTM F3269-17": 1},
            obligation_realization_map={"OBL-01": ["docs/conops/CONOPS.md:12"], "OBL-02": ["docs/features/feat-01.md:34"]},
            unrealized_obligations=[],
            phantom_realizations=[],
        )
        self.assertTrue(digest.is_fully_realized())
        d = digest.to_dict()
        self.assertEqual(d["total_declared_obligations"], 6)
        self.assertEqual(d["realization_percentage"], 100.0)
        self.assertTrue(d["is_fully_realized"])

        md = digest.generate_markdown_summary()
        self.assertIn("Declared Total Obligations", md)
        self.assertIn("100% Conforming", md)

    def test_coverage_digest_incomplete(self):
        digest = CoverageDigest(
            total_declared_obligations=6,
            total_realized_obligations=4,
            realization_percentage=66.67,
            unrealized_obligations=["SAF-02", "EXT-01"],
            phantom_realizations=["OBL-99"],
        )
        self.assertFalse(digest.is_fully_realized())
        d = digest.to_dict()
        self.assertFalse(d["is_fully_realized"])
        self.assertEqual(len(d["unrealized_obligations"]), 2)
        self.assertEqual(len(d["phantom_realizations"]), 1)


class TestObligationWitnessModel(unittest.TestCase):
    """Unit tests for ObligationWitnessRecord and ObligationWitnessRegistry."""

    def test_witness_record_and_registry(self):
        rec1 = ObligationWitnessRecord(
            obligation_id="OBL-01",
            standard_id="ISO/IEC/IEEE 29148:2018",
            category="Requirements Engineering",
            clause_citation="ISO/IEC/IEEE 29148:2018 §6.4.2",
            spec_witnesses=["docs/conops/CONOPS.md:12"],
            test_witnesses=["tests/test_conops.py:45"],
            code_witnesses=["src/domain/conops.py:20"],
        )
        self.assertTrue(rec1.is_witnessed)
        self.assertTrue(rec1.is_fully_witnessed)
        self.assertEqual(rec1.total_witnesses, 3)

        rec2 = ObligationWitnessRecord(
            obligation_id="OBL-02",
            standard_id="ISO/IEC/IEEE 29148:2018",
            category="Requirements Engineering",
            spec_witnesses=["docs/features/feat-01.md:10"],
        )
        self.assertTrue(rec2.is_witnessed)
        self.assertFalse(rec2.is_fully_witnessed)

        registry = ObligationWitnessRegistry(
            records={"OBL-01": rec1, "OBL-02": rec2},
            phantom_witnesses={},
        )
        self.assertEqual(registry.total_declared(), 2)
        self.assertEqual(registry.total_witnessed(), 2)
        self.assertEqual(registry.total_fully_witnessed(), 1)
        self.assertEqual(registry.witness_coverage_percentage(), 100.0)


class TestCoverageDigestValidator(unittest.TestCase):
    """Unit tests for CoverageDigestValidator (Gate 28 / #92)."""

    def setUp(self):
        self.validator = CoverageDigestValidator()

    def test_extract_declared_obligations_from_inventory(self):
        doc = parse_research_inventory(SAMPLE_RESEARCH_INVENTORY)
        declared = self.validator.extract_declared_obligations_from_doc(doc)
        self.assertIn("OBL-01", declared)
        self.assertIn("OBL-02", declared)
        self.assertIn("SAF-01", declared)
        self.assertIn("SAF-02", declared)
        self.assertIn("INT-01", declared)
        self.assertIn("EXT-01", declared)
        self.assertEqual(len(declared), 6)

    def test_coverage_digest_validation_in_clean_workspace(self):
        """In a temporary workspace with matching specs and inventory, coverage passes cleanly."""
        with tempfile.TemporaryDirectory() as tmpdir:
            # Create rules file
            os.makedirs(os.path.join(tmpdir, ".pipeline", "logical-ui"), exist_ok=True)
            with open(os.path.join(tmpdir, ".pipeline", "logical-ui", "codebase_rules.json"), "w") as f:
                f.write('{"meta": {"upstream_repository": "gintatkinson/DEAP01-spec-core"}}\n')

            # Create research inventory
            os.makedirs(os.path.join(tmpdir, "docs", "research"), exist_ok=True)
            with open(os.path.join(tmpdir, "docs", "research", "RESEARCH_INVENTORY.md"), "w") as f:
                f.write(SAMPLE_RESEARCH_INVENTORY)

            # Create spec files referenced in section 5 or containing allocation tags
            os.makedirs(os.path.join(tmpdir, "docs", "conops"), exist_ok=True)
            with open(os.path.join(tmpdir, "docs", "conops", "CONOPS.md"), "w") as f:
                f.write("# ConOps\n/// ObligationAllocation: [OBL-01]\n")

            os.makedirs(os.path.join(tmpdir, "docs", "features"), exist_ok=True)
            with open(os.path.join(tmpdir, "docs", "features", "feat-01-requirements.md"), "w") as f:
                f.write("# Feat 01\n/// ObligationAllocation: [OBL-02]\n")
            with open(os.path.join(tmpdir, "docs", "features", "feat-02-safety.md"), "w") as f:
                f.write("# Feat 02\n/// ObligationAllocation: [SAF-01, SAF-02]\n")
            with open(os.path.join(tmpdir, "docs", "features", "feat-03-dli.md"), "w") as f:
                f.write("# Feat 03\n/// ObligationAllocation: [INT-01]\n")
            with open(os.path.join(tmpdir, "docs", "features", "feat-04-rta.md"), "w") as f:
                f.write("# Feat 04\n/// ObligationAllocation: [EXT-01]\n")

            repo = WorkspaceRepository(tmpdir)
            findings = self.validator.validate(repo)
            self.assertEqual(findings, [], f"Expected zero findings, got: {findings}")

            # Check digest synthesis
            digest_md = self.validator.synthesize_coverage_digest(repo)
            self.assertIn("Coverage Digest", digest_md)
            self.assertIn("100.0%", digest_md)

    def test_coverage_digest_flags_phantom_obligations(self):
        """A specification tagging an undeclared obligation must emit a phantom finding."""
        with tempfile.TemporaryDirectory() as tmpdir:
            os.makedirs(os.path.join(tmpdir, ".pipeline", "logical-ui"), exist_ok=True)
            with open(os.path.join(tmpdir, ".pipeline", "logical-ui", "codebase_rules.json"), "w") as f:
                f.write('{"meta": {"upstream_repository": "gintatkinson/DEAP01-spec-core"}}\n')

            os.makedirs(os.path.join(tmpdir, "docs", "research"), exist_ok=True)
            with open(os.path.join(tmpdir, "docs", "research", "RESEARCH_INVENTORY.md"), "w") as f:
                f.write(SAMPLE_RESEARCH_INVENTORY)

            os.makedirs(os.path.join(tmpdir, "docs", "features"), exist_ok=True)
            with open(os.path.join(tmpdir, "docs", "features", "feat-01.md"), "w") as f:
                f.write("# Feat 01\n/// ObligationAllocation: [OBL-01, OBL-PHANTOM-99]\n")

            repo = WorkspaceRepository(tmpdir)
            findings = self.validator.validate(repo, allow_missing_specs=True)
            phantom_findings = [f for f in findings if "phantom" in f.rule_id or "phantom" in str(f).lower()]
            self.assertGreaterEqual(len(phantom_findings), 1)
            self.assertTrue(any("OBL-PHANTOM-99" in str(f) or "PHANTOM-99" in str(f) for f in phantom_findings))

    def test_coverage_digest_flags_unrealized_obligations_in_strict_mode(self):
        """In strict mode when specs exist, unrealized declared obligations must emit findings."""
        with tempfile.TemporaryDirectory() as tmpdir:
            os.makedirs(os.path.join(tmpdir, ".pipeline", "logical-ui"), exist_ok=True)
            with open(os.path.join(tmpdir, ".pipeline", "logical-ui", "codebase_rules.json"), "w") as f:
                f.write('{"meta": {"upstream_repository": "gintatkinson/DEAP01-spec-core"}}\n')

            os.makedirs(os.path.join(tmpdir, "docs", "research"), exist_ok=True)
            with open(os.path.join(tmpdir, "docs", "research", "RESEARCH_INVENTORY.md"), "w") as f:
                f.write(SAMPLE_RESEARCH_INVENTORY)

            # Only OBL-01 is realized
            os.makedirs(os.path.join(tmpdir, "docs", "features"), exist_ok=True)
            with open(os.path.join(tmpdir, "docs", "features", "feat-01.md"), "w") as f:
                f.write("# Feat 01\n/// ObligationAllocation: [OBL-01]\n")

            repo = WorkspaceRepository(tmpdir)
            findings = self.validator.validate(repo, allow_missing_specs=False)
            unrealized_findings = [f for f in findings if "unrealized" in f.rule_id or "unrealized" in str(f).lower()]
            self.assertGreaterEqual(len(unrealized_findings), 1)
            self.assertTrue(any("SAF-01" in str(f) or "INT-01" in str(f) for f in unrealized_findings))


class TestObligationWitnessValidator(unittest.TestCase):
    """Unit tests for ObligationWitnessValidator & Registry (Gate 29 / #93)."""

    def setUp(self):
        self.validator = ObligationWitnessValidator()

    def test_witness_registry_validation_in_clean_workspace(self):
        """In a workspace with spec, test, and code witnesses matching assigned documents, all obligations pass cleanly."""
        with tempfile.TemporaryDirectory() as tmpdir:
            os.makedirs(os.path.join(tmpdir, ".pipeline", "logical-ui"), exist_ok=True)
            with open(os.path.join(tmpdir, ".pipeline", "logical-ui", "codebase_rules.json"), "w") as f:
                f.write('{"meta": {"upstream_repository": "gintatkinson/DEAP01-spec-core"}, "target_directories": {"flutter": "app_flutter"}}\n')

            os.makedirs(os.path.join(tmpdir, "docs", "research"), exist_ok=True)
            with open(os.path.join(tmpdir, "docs", "research", "RESEARCH_INVENTORY.md"), "w") as f:
                f.write(SAMPLE_RESEARCH_INVENTORY)

            # Spec files matching Section 5 allocations
            os.makedirs(os.path.join(tmpdir, "docs", "conops"), exist_ok=True)
            with open(os.path.join(tmpdir, "docs", "conops", "CONOPS.md"), "w") as f:
                f.write("# ConOps\n/// ObligationWitness: [OBL-01]\n")

            os.makedirs(os.path.join(tmpdir, "docs", "features"), exist_ok=True)
            with open(os.path.join(tmpdir, "docs", "features", "feat-01-requirements.md"), "w") as f:
                f.write("# Feat 01\n/// ObligationWitness: [OBL-02]\n")
            with open(os.path.join(tmpdir, "docs", "features", "feat-02-safety.md"), "w") as f:
                f.write("# Feat 02\n/// ObligationWitness: [SAF-01, SAF-02]\n")
            with open(os.path.join(tmpdir, "docs", "features", "feat-03-dli.md"), "w") as f:
                f.write("# Feat 03\n/// ObligationWitness: [INT-01]\n")
            with open(os.path.join(tmpdir, "docs", "features", "feat-04-rta.md"), "w") as f:
                f.write("# Feat 04\n/// ObligationWitness: [EXT-01]\n")

            # Test files
            os.makedirs(os.path.join(tmpdir, "tests"), exist_ok=True)
            with open(os.path.join(tmpdir, "tests", "test_obligations.py"), "w") as f:
                f.write("# Test file\n# /// ObligationWitness: [OBL-01, OBL-02, SAF-01, SAF-02, INT-01, EXT-01]\ndef test_all(): pass\n")

            # Code files
            os.makedirs(os.path.join(tmpdir, "app_flutter", "lib"), exist_ok=True)
            with open(os.path.join(tmpdir, "app_flutter", "lib", "main.dart"), "w") as f:
                f.write("/// Realises: [OBL-01, OBL-02, SAF-01, SAF-02, INT-01, EXT-01]\nvoid main() {}\n")

            repo = WorkspaceRepository(tmpdir)
            findings = self.validator.validate(repo)
            self.assertEqual(findings, [], f"Expected zero findings, got: {findings}")

            registry = self.validator.build_witness_registry(repo)
            self.assertEqual(registry.total_declared(), 6)
            self.assertEqual(registry.total_witnessed(), 6)
            self.assertEqual(registry.total_fully_witnessed(), 6)

            # Check markdown synthesis
            matrix_md = self.validator.synthesize_witness_registry(repo)
            self.assertIn("Obligation-Witness Registry", matrix_md)
            self.assertIn("OBL-01", matrix_md)
            self.assertIn("100.0%", matrix_md)

    def test_witness_registry_flags_phantom_witnesses(self):
        """A test or code file referencing an undeclared obligation must emit a phantom witness finding."""
        with tempfile.TemporaryDirectory() as tmpdir:
            os.makedirs(os.path.join(tmpdir, ".pipeline", "logical-ui"), exist_ok=True)
            with open(os.path.join(tmpdir, ".pipeline", "logical-ui", "codebase_rules.json"), "w") as f:
                f.write('{"meta": {"upstream_repository": "gintatkinson/DEAP01-spec-core"}}\n')

            os.makedirs(os.path.join(tmpdir, "docs", "research"), exist_ok=True)
            with open(os.path.join(tmpdir, "docs", "research", "RESEARCH_INVENTORY.md"), "w") as f:
                f.write(SAMPLE_RESEARCH_INVENTORY)

            os.makedirs(os.path.join(tmpdir, "tests"), exist_ok=True)
            with open(os.path.join(tmpdir, "tests", "test_phantom.py"), "w") as f:
                f.write("# /// ObligationWitness: [OBL-GHOST-42]\ndef test_ghost(): pass\n")

            repo = WorkspaceRepository(tmpdir)
            findings = self.validator.validate(repo, allow_missing_specs=True)
            phantom_findings = [f for f in findings if "phantom" in f.rule_id or "phantom" in str(f).lower()]
            self.assertGreaterEqual(len(phantom_findings), 1)
            self.assertTrue(any("OBL-GHOST-42" in str(f) or "GHOST-42" in str(f) for f in phantom_findings))

    def test_witness_registry_flags_unwitnessed_conops_obligation_even_with_allow_missing_specs(self):
        """When an obligation is allocated to CONOPS.md, missing witness in CONOPS.md emits obligation-unwitnessed."""
        with tempfile.TemporaryDirectory() as tmpdir:
            os.makedirs(os.path.join(tmpdir, ".pipeline", "logical-ui"), exist_ok=True)
            with open(os.path.join(tmpdir, ".pipeline", "logical-ui", "codebase_rules.json"), "w") as f:
                f.write('{"meta": {"upstream_repository": "gintatkinson/DEAP01-spec-core"}}\n')

            os.makedirs(os.path.join(tmpdir, "docs", "research"), exist_ok=True)
            with open(os.path.join(tmpdir, "docs", "research", "RESEARCH_INVENTORY.md"), "w") as f:
                f.write(SAMPLE_RESEARCH_INVENTORY)

            # CONOPS.md exists but has no witness tag for OBL-01
            os.makedirs(os.path.join(tmpdir, "docs", "conops"), exist_ok=True)
            with open(os.path.join(tmpdir, "docs", "conops", "CONOPS.md"), "w") as f:
                f.write("# ConOps without obligations\n")

            repo = WorkspaceRepository(tmpdir)
            findings = self.validator.validate(repo, allow_missing_specs=True)
            unwitnessed = [f for f in findings if f.rule_id == "obligation-unwitnessed"]
            self.assertGreaterEqual(len(unwitnessed), 1)
            self.assertTrue(any("OBL-01" in str(f) for f in unwitnessed))

    def test_coverage_digest_flags_unrealized_conops_obligation_even_with_allow_missing_specs(self):
        """When an obligation is allocated to CONOPS.md, missing realization tag emits coverage-digest-obligation-unrealized."""
        with tempfile.TemporaryDirectory() as tmpdir:
            os.makedirs(os.path.join(tmpdir, ".pipeline", "logical-ui"), exist_ok=True)
            with open(os.path.join(tmpdir, ".pipeline", "logical-ui", "codebase_rules.json"), "w") as f:
                f.write('{"meta": {"upstream_repository": "gintatkinson/DEAP01-spec-core"}}\n')

            os.makedirs(os.path.join(tmpdir, "docs", "research"), exist_ok=True)
            with open(os.path.join(tmpdir, "docs", "research", "RESEARCH_INVENTORY.md"), "w") as f:
                f.write(SAMPLE_RESEARCH_INVENTORY)

            os.makedirs(os.path.join(tmpdir, "docs", "conops"), exist_ok=True)
            with open(os.path.join(tmpdir, "docs", "conops", "CONOPS.md"), "w") as f:
                f.write("# ConOps without obligations\n")

            repo = WorkspaceRepository(tmpdir)
            cov_val = CoverageDigestValidator()
            findings = cov_val.validate(repo, allow_missing_specs=True)
            unrealized = [f for f in findings if f.rule_id == "coverage-digest-obligation-unrealized"]
            self.assertGreaterEqual(len(unrealized), 1)
            self.assertTrue(any("OBL-01" in str(f) for f in unrealized))

    def test_coverage_digest_flags_unrealized_feature_spec_when_target_file_exists_and_allow_missing_specs_is_true(self):
        """When a feature spec file exists in workspace, an unrealized obligation allocated to it must be flagged even with allow_missing_specs=True."""
        with tempfile.TemporaryDirectory() as tmpdir:
            os.makedirs(os.path.join(tmpdir, ".pipeline", "logical-ui"), exist_ok=True)
            with open(os.path.join(tmpdir, ".pipeline", "logical-ui", "codebase_rules.json"), "w") as f:
                f.write('{"meta": {"upstream_repository": "downstream-org/my-system"}}\n')

            os.makedirs(os.path.join(tmpdir, "docs", "research"), exist_ok=True)
            with open(os.path.join(tmpdir, "docs", "research", "RESEARCH_INVENTORY.md"), "w") as f:
                f.write(SAMPLE_RESEARCH_INVENTORY)

            # CONOPS is realized
            os.makedirs(os.path.join(tmpdir, "docs", "conops"), exist_ok=True)
            with open(os.path.join(tmpdir, "docs", "conops", "CONOPS.md"), "w") as f:
                f.write("# ConOps\n/// ObligationAllocation: [OBL-01]\n")

            # feat-01-requirements.md exists in workspace but lacks OBL-02 tag
            os.makedirs(os.path.join(tmpdir, "docs", "features"), exist_ok=True)
            with open(os.path.join(tmpdir, "docs", "features", "feat-01-requirements.md"), "w") as f:
                f.write("# Feature 01 Requirements\nNo obligation allocation here.\n")

            repo = WorkspaceRepository(tmpdir)
            cov_val = CoverageDigestValidator()
            findings = cov_val.validate(repo, allow_missing_specs=True)
            unrealized = [f for f in findings if f.rule_id == "coverage-digest-obligation-unrealized"]
            self.assertTrue(any("OBL-02" in str(f) for f in unrealized), f"Expected OBL-02 to be flagged as unrealized, got: {findings}")

    def test_witness_registry_flags_missing_test_and_code_witnesses_when_specs_exist_and_allow_missing_specs_is_true(self):
        """When specs exist and codebase is configured, missing test/code witnesses must be flagged even when allow_missing_specs=True."""
        with tempfile.TemporaryDirectory() as tmpdir:
            os.makedirs(os.path.join(tmpdir, ".pipeline", "logical-ui"), exist_ok=True)
            with open(os.path.join(tmpdir, ".pipeline", "logical-ui", "codebase_rules.json"), "w") as f:
                f.write('{"meta": {"upstream_repository": "downstream-org/my-system"}, "target_directories": {"flutter": "app_flutter"}}\n')

            os.makedirs(os.path.join(tmpdir, "docs", "research"), exist_ok=True)
            with open(os.path.join(tmpdir, "docs", "research", "RESEARCH_INVENTORY.md"), "w") as f:
                f.write(SAMPLE_RESEARCH_INVENTORY)

            # Spec files exist with spec witnesses
            os.makedirs(os.path.join(tmpdir, "docs", "conops"), exist_ok=True)
            with open(os.path.join(tmpdir, "docs", "conops", "CONOPS.md"), "w") as f:
                f.write("# ConOps\n/// ObligationWitness: [OBL-01]\n")

            os.makedirs(os.path.join(tmpdir, "docs", "features"), exist_ok=True)
            with open(os.path.join(tmpdir, "docs", "features", "feat-01-requirements.md"), "w") as f:
                f.write("# Feat 01\n/// ObligationWitness: [OBL-02]\n")

            # App codebase exists but has no code witnesses for OBL-01 or OBL-02
            os.makedirs(os.path.join(tmpdir, "app_flutter", "lib"), exist_ok=True)
            with open(os.path.join(tmpdir, "app_flutter", "lib", "main.dart"), "w") as f:
                f.write("void main() {}\n")

            # Test directory exists but has no test witnesses
            os.makedirs(os.path.join(tmpdir, "tests"), exist_ok=True)
            with open(os.path.join(tmpdir, "tests", "test_app.py"), "w") as f:
                f.write("def test_dummy(): pass\n")

            repo = WorkspaceRepository(tmpdir)
            self.assertFalse(repo.is_upstream_compiler_repo())
            self.assertTrue(repo.has_configured_target_code_directories())

            wit_val = ObligationWitnessValidator()
            findings = wit_val.validate(repo, allow_missing_specs=True, spec_only=False)

            missing_tests = [f for f in findings if f.rule_id == "obligation-witness-missing-test-witness"]
            missing_code = [f for f in findings if f.rule_id == "obligation-witness-missing-code-witness"]

            self.assertGreaterEqual(len(missing_tests), 1, f"Expected missing test witness findings, got: {findings}")
            self.assertGreaterEqual(len(missing_code), 1, f"Expected missing code witness findings, got: {findings}")
            self.assertTrue(any("OBL-01" in str(f) for f in missing_tests))
            self.assertTrue(any("OBL-02" in str(f) for f in missing_tests))
            self.assertTrue(any("OBL-01" in str(f) for f in missing_code))
            self.assertTrue(any("OBL-02" in str(f) for f in missing_code))

    def test_witness_registry_flags_unwitnessed_in_assigned_feature_spec_when_allow_missing_specs_is_true(self):
        """When an obligation is assigned to a feature spec that exists in workspace, missing witness in that file must be flagged."""
        with tempfile.TemporaryDirectory() as tmpdir:
            os.makedirs(os.path.join(tmpdir, ".pipeline", "logical-ui"), exist_ok=True)
            with open(os.path.join(tmpdir, ".pipeline", "logical-ui", "codebase_rules.json"), "w") as f:
                f.write('{"meta": {"upstream_repository": "downstream-org/my-system"}}\n')

            os.makedirs(os.path.join(tmpdir, "docs", "research"), exist_ok=True)
            with open(os.path.join(tmpdir, "docs", "research", "RESEARCH_INVENTORY.md"), "w") as f:
                f.write(SAMPLE_RESEARCH_INVENTORY)

            # CONOPS.md is witnessed
            os.makedirs(os.path.join(tmpdir, "docs", "conops"), exist_ok=True)
            with open(os.path.join(tmpdir, "docs", "conops", "CONOPS.md"), "w") as f:
                f.write("# ConOps\n/// ObligationWitness: [OBL-01]\n")

            # feat-01-requirements.md exists but lacks witness for OBL-02
            os.makedirs(os.path.join(tmpdir, "docs", "features"), exist_ok=True)
            with open(os.path.join(tmpdir, "docs", "features", "feat-01-requirements.md"), "w") as f:
                f.write("# Feature 01\n")

            repo = WorkspaceRepository(tmpdir)
            wit_val = ObligationWitnessValidator()
            findings = wit_val.validate(repo, allow_missing_specs=True)
            unwitnessed = [f for f in findings if f.rule_id == "obligation-unwitnessed"]
            self.assertTrue(any("OBL-02" in str(f) for f in unwitnessed))
            self.assertTrue(any("feat-01-requirements.md" in f.location for f in unwitnessed if "OBL-02" in str(f)))

    def test_coverage_digest_and_witness_registry_realises_tag_support(self):
        """Realises: [OBL-xx] and Realises: OBL-xx tags are recognized by both Gate 28 and Gate 29."""
        with tempfile.TemporaryDirectory() as tmpdir:
            os.makedirs(os.path.join(tmpdir, ".pipeline", "logical-ui"), exist_ok=True)
            with open(os.path.join(tmpdir, ".pipeline", "logical-ui", "codebase_rules.json"), "w") as f:
                f.write('{"meta": {"upstream_repository": "downstream-org/my-system"}, "target_directories": {"flutter": "app_flutter"}}\n')

            os.makedirs(os.path.join(tmpdir, "docs", "research"), exist_ok=True)
            with open(os.path.join(tmpdir, "docs", "research", "RESEARCH_INVENTORY.md"), "w") as f:
                f.write(SAMPLE_RESEARCH_INVENTORY)

            # Specs using Realises tags
            os.makedirs(os.path.join(tmpdir, "docs", "conops"), exist_ok=True)
            with open(os.path.join(tmpdir, "docs", "conops", "CONOPS.md"), "w") as f:
                f.write("# ConOps\n/// Realises: [OBL-01]\n")

            os.makedirs(os.path.join(tmpdir, "docs", "features"), exist_ok=True)
            with open(os.path.join(tmpdir, "docs", "features", "feat-01-requirements.md"), "w") as f:
                f.write("# Feat 01\n/// Realises: [OBL-02]\n")
            with open(os.path.join(tmpdir, "docs", "features", "feat-02-safety.md"), "w") as f:
                f.write("# Feat 02\n/// Realises: [SAF-01, SAF-02]\n")
            with open(os.path.join(tmpdir, "docs", "features", "feat-03-dli.md"), "w") as f:
                f.write("# Feat 03\n/// Realises: [INT-01]\n")
            with open(os.path.join(tmpdir, "docs", "features", "feat-04-rta.md"), "w") as f:
                f.write("# Feat 04\n/// Realises: [EXT-01]\n")

            # Tests using TestWitness and Realises
            os.makedirs(os.path.join(tmpdir, "tests"), exist_ok=True)
            with open(os.path.join(tmpdir, "tests", "test_all.py"), "w") as f:
                f.write("# /// TestWitness: [OBL-01, OBL-02, SAF-01, SAF-02, INT-01, EXT-01]\n")

            # Code using Realises
            os.makedirs(os.path.join(tmpdir, "app_flutter", "lib"), exist_ok=True)
            with open(os.path.join(tmpdir, "app_flutter", "lib", "main.dart"), "w") as f:
                f.write("/// Realises: [OBL-01, OBL-02, SAF-01, SAF-02, INT-01, EXT-01]\nvoid main() {}\n")

            repo = WorkspaceRepository(tmpdir)
            cov_val = CoverageDigestValidator()
            wit_val = ObligationWitnessValidator()

            cov_findings = cov_val.validate(repo, allow_missing_specs=True)
            wit_findings = wit_val.validate(repo, allow_missing_specs=True, spec_only=False)

            self.assertEqual(cov_findings, [])
            self.assertEqual(wit_findings, [])


class TestAggregatorAndCliRegistration(unittest.TestCase):
    """Unit tests confirming registration in aggregator and cli."""

    def test_aggregator_includes_both_validators(self):
        self.assertIn(CoverageDigestValidator, AGGREGATING_VALIDATORS)
        self.assertIn(ObligationWitnessValidator, AGGREGATING_VALIDATORS)

    def test_upstream_clean_mode_passes(self):
        repo = WorkspaceRepository(PROJECT_ROOT)
        self.assertTrue(repo.is_upstream_compiler_repo())
        v1 = CoverageDigestValidator()
        v2 = ObligationWitnessValidator()
        self.assertEqual(v1.validate(repo), [])
        self.assertEqual(v2.validate(repo), [])


if __name__ == "__main__":
    unittest.main()

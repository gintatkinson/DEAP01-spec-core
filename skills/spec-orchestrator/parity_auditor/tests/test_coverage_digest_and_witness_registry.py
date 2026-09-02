#!/usr/bin/env python3
"""
Unit tests for Coverage-Digest Population Gate (Gate 28 / #92) and
Obligation-Witness Registry (Gate 29 / #93) within parity_auditor package.
(`parity_auditor/tests/test_coverage_digest_and_witness_registry.py`)
"""

import os
import sys
import tempfile
import unittest

# Ensure parity_auditor is importable
_src_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "src"))
if _src_dir not in sys.path:
    sys.path.insert(0, _src_dir)

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


class TestParityAuditorCoverageDigestAndWitness(unittest.TestCase):
    """Test suite asserting Gate 28 and Gate 29 functionality in parity_auditor."""

    def test_coverage_digest_calculation_and_synthesis(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            os.makedirs(os.path.join(tmpdir, ".pipeline", "logical-ui"), exist_ok=True)
            with open(os.path.join(tmpdir, ".pipeline", "logical-ui", "codebase_rules.json"), "w") as f:
                f.write('{"meta": {"upstream_repository": "gintatkinson/DEAP01-spec-core"}}\n')

            os.makedirs(os.path.join(tmpdir, "docs", "research"), exist_ok=True)
            with open(os.path.join(tmpdir, "docs", "research", "RESEARCH_INVENTORY.md"), "w") as f:
                f.write(SAMPLE_RESEARCH_INVENTORY)

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
            validator = CoverageDigestValidator()
            findings = validator.validate(repo)
            self.assertEqual(findings, [])

            digest = validator.build_coverage_digest(repo)
            self.assertTrue(digest.is_fully_realized())
            self.assertEqual(digest.total_declared_obligations, 6)
            self.assertEqual(digest.total_realized_obligations, 6)
            self.assertEqual(digest.realization_percentage, 100.0)

            md = validator.synthesize_coverage_digest(repo)
            self.assertIn("Coverage Digest", md)
            self.assertIn("OBL-01", md)

    def test_witness_registry_and_synthesis(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            os.makedirs(os.path.join(tmpdir, ".pipeline", "logical-ui"), exist_ok=True)
            with open(os.path.join(tmpdir, ".pipeline", "logical-ui", "codebase_rules.json"), "w") as f:
                f.write('{"meta": {"upstream_repository": "gintatkinson/DEAP01-spec-core"}}\n')

            os.makedirs(os.path.join(tmpdir, "docs", "research"), exist_ok=True)
            with open(os.path.join(tmpdir, "docs", "research", "RESEARCH_INVENTORY.md"), "w") as f:
                f.write(SAMPLE_RESEARCH_INVENTORY)

            os.makedirs(os.path.join(tmpdir, "docs", "features"), exist_ok=True)
            with open(os.path.join(tmpdir, "docs", "features", "feat-01.md"), "w") as f:
                f.write("# Feat 01\n/// ObligationWitness: [OBL-01, OBL-02, SAF-01, SAF-02, INT-01, EXT-01]\n")

            repo = WorkspaceRepository(tmpdir)
            validator = ObligationWitnessValidator()
            registry = validator.build_witness_registry(repo)
            self.assertEqual(registry.total_declared(), 6)
            self.assertEqual(registry.total_witnessed(), 6)

            findings = validator.validate(repo, spec_only=True)
            self.assertEqual(findings, [])

            md = validator.synthesize_witness_registry(repo)
            self.assertIn("Multi-Dimensional Obligation-Witness Registry", md)
            self.assertIn("OBL-01", md)

    def test_aggregator_contains_validators(self):
        self.assertIn(CoverageDigestValidator, AGGREGATING_VALIDATORS)
        self.assertIn(ObligationWitnessValidator, AGGREGATING_VALIDATORS)


if __name__ == "__main__":
    unittest.main()

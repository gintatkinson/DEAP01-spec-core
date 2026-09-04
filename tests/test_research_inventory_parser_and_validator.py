#!/usr/bin/env python3
"""
Unit tests for Cited Research Inventory and Declared-Total Population Register (#97).
(`tests/test_research_inventory_parser_and_validator.py`)

Verifies:
1. Canonical template at `skills/spec-orchestrator/resources/RESEARCH_INVENTORY_CANONICAL_TEMPLATE.md`
   and `.agents/skills/spec-orchestrator/resources/RESEARCH_INVENTORY_CANONICAL_TEMPLATE.md`
   contains exact CommonMark tables for:
   - Normative Standards & Baseline Documents Inventory (with schema: Standard / Baseline ID, Issuing Body, Title, Applicable Clauses, Obligation Category, Declared Total, Clause Citation)
   - Declared-Total Population Register (Category, Standard ID, Target Metric / Obligation Count, Verification Mechanism, Public Clause Citation)
   - External Additions & Domain Extensions Registry (all clause-cited)
2. Data models & parser in parity_auditor:
   - Extracts all standards, population entries, external additions, metadata, and allocations
   - Extracts and calculates declared totals per standard and per category
3. Clause citation validator:
   - Validates formal public clause citations
   - Rejects un-cited or invalid additions (empty, TBD, N/A, unknown, missing clause specifier)
   - Asserts declared-total calculations and schema header conformance
"""

import os
import re
import sys
import unittest

PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if PROJECT_ROOT not in sys.path:
    sys.path.insert(0, PROJECT_ROOT)

PARITY_SRC = os.path.join(PROJECT_ROOT, "skills", "spec-orchestrator", "parity_auditor", "src")
if PARITY_SRC not in sys.path:
    sys.path.insert(0, PARITY_SRC)

from parity_auditor.parsers.research_inventory import (
    ResearchInventoryParser,
    parse_research_inventory,
    is_valid_public_clause_citation,
)
from parity_auditor.core.models import (
    NormativeStandard,
    PopulationRegisterEntry,
    ExternalAdditionEntry,
    ResearchInventoryDocument,
)
from parity_auditor.validators.research_inventory_validator import (
    ResearchInventoryValidator,
)
from parity_auditor.core.workspace import WorkspaceRepository


CANONICAL_TEMPLATE_PATH = os.path.join(
    PROJECT_ROOT, "skills", "spec-orchestrator", "resources", "RESEARCH_INVENTORY_CANONICAL_TEMPLATE.md"
)
MIRROR_TEMPLATE_PATH = os.path.join(
    PROJECT_ROOT, ".agents", "skills", "spec-orchestrator", "resources", "RESEARCH_INVENTORY_CANONICAL_TEMPLATE.md"
)


SAMPLE_VALID_RESEARCH_INVENTORY = r"""
| Attribute | Value |
| :--- | :--- |
| **Title** | Cited Research Inventory: Test System |
| **Version** | 1.0.0 |
| **Date** | 2026-09-02 |

# Cited Research Inventory: Test System

## 1. Scope & System Identification
- **System Identifier:** `TEST-SYS`
- **Operational Domain:** `Autonomous Systems`
- **Research Scope:** Comprehensive baseline.
- **Applicability Statement:** Applicable across all sub-systems.

## 2. Normative Standards & Baseline Documents Inventory
| Standard / Baseline ID | Issuing Body | Title | Applicable Clauses | Obligation Category | Declared Total | Clause Citation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| ISO/IEC/IEEE 29148:2018 | ISO/IEC/IEEE | Systems and Software Engineering — Requirements Engineering | §6.4.2 ConOps, §6.4.3 OpsCon, §8.4 System Requirements | Requirements Engineering | 3 | ISO/IEC/IEEE 29148:2018 §6.4.2, §6.4.3, §8.4 |
| NATO STANAG 4586 | NATO | Standard Interfaces of Autonomous Control Systems | Interoperability Profiles & DLI/VCI Interfaces | Interoperability | 2 | STANAG 4586 Ed. 4 §3.2, §4.1 |
| RTCA DO-178C / DO-254 | RTCA / EUROCAE | Software and Electronic Hardware Considerations in Airborne Systems | §6.3 Software Architecture, §11.0 Software Life Cycle Data | Safety Assurance | 2 | DO-178C §6.3, DO-254 §11.0 |
| MIL-STD-882E | DoD | System Safety | Task 201 Preliminary Hazard Analysis, Task 205 System Hazard Analysis | Hazard Analysis | 2 | MIL-STD-882E Task 201, Task 205 |

## 3. Declared-Total Population Register
| Obligation ID | Category | Standard ID | Target Metric / Obligation Count | Verification Mechanism | Public Clause Citation |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `OBL-01` | Normative Obligation | ISO/IEC/IEEE 29148:2018 | 2 | Inspection & Traceability Audit | ISO/IEC/IEEE 29148:2018 §6.4.2 |
| `OBL-02` | Normative Obligation | ISO/IEC/IEEE 29148:2018 | 1 | Schema Validation | ISO/IEC/IEEE 29148:2018 §8.4 |
| `INT-01` | Interoperability | NATO STANAG 4586 | 2 | Interface Conformance Test | STANAG 4586 Ed. 4 §3.2 |
| `SAF-01` | Safety Constraint | RTCA DO-178C / DO-254 | 2 | Formal Verification & Static Analysis | DO-178C §6.3 |
| `HAZ-01` | Hazard Analysis | MIL-STD-882E | 2 | System Hazard Analysis | MIL-STD-882E Task 201 |

## 4. External Additions & Domain Extensions Registry
| Extension ID | Category | Standard / Baseline ID | Declared Total | Target Metric / Obligation Count | Verification Mechanism | Public Clause Citation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `EXT-01` | Domain Extension | ASTM F3269-17 | 1 | 1 | Automated Flight Bounds Test | ASTM F3269-17 §5.2 |
| `EXT-02` | External Addition | RTCA DO-365B | 2 | 2 | DAA Simulation & Radar Bench | RTCA DO-365B §2.2.4 |

## 5. Clause-Level Allocation & Traceability Matrix
| Population ID | Standard ID | Clause Citation | Clause Title / Requirement Excerpt | Specification Phase | Downstream Spec File / Tag |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `OBL-01` | ISO/IEC/IEEE 29148:2018 | ISO/IEC/IEEE 29148:2018 §6.4.2 | System ConOps Requirements | Phase 1 (Structural) | `docs/features/feat-01.md` |

## 6. Normative Completeness & Gap Analysis
| Metric Parameter | Value | Target Threshold | Compliance Status |
| :--- | :--- | :--- | :--- |
| Declared Total Normative Obligations | 9 | $\ge 1$ | Conforming |
| Declared Total Safety Constraints | 2 | $\ge 1$ | Conforming |
| Clause Citation Traceability Percentage | 100% | 100% | Conforming |
| Un-Cited / Speculative Additions | 0 | 0 (Strict Zero Tolerance) | Conforming |
"""


class TestResearchInventoryTemplate(unittest.TestCase):
    """Test suite asserting the canonical template structure and tables."""

    def test_canonical_template_exists_and_mirrored(self):
        """Verify that RESEARCH_INVENTORY_CANONICAL_TEMPLATE.md exists and is mirrored in .agents/."""
        self.assertTrue(os.path.isfile(CANONICAL_TEMPLATE_PATH), f"Missing template: {CANONICAL_TEMPLATE_PATH}")
        self.assertTrue(os.path.isfile(MIRROR_TEMPLATE_PATH), f"Missing mirror template: {MIRROR_TEMPLATE_PATH}")

        with open(CANONICAL_TEMPLATE_PATH, "r", encoding="utf-8") as f1, open(MIRROR_TEMPLATE_PATH, "r", encoding="utf-8") as f2:
            self.assertEqual(f1.read(), f2.read(), "Canonical template and .agents mirror must be identical")

    def test_canonical_template_table_headers_normative_standards(self):
        """Verify Section 2 table contains exact 7-column schema."""
        with open(CANONICAL_TEMPLATE_PATH, "r", encoding="utf-8") as f:
            content = f.read()

        self.assertIn("Normative Standards & Baseline Documents Inventory", content)
        expected_headers = [
            "Standard / Baseline ID",
            "Issuing Body",
            "Title",
            "Applicable Clauses",
            "Obligation Category",
            "Declared Total",
            "Clause Citation",
        ]
        for header in expected_headers:
            self.assertIn(header, content, f"Missing header '{header}' in Section 2 table")

    def test_canonical_template_table_headers_population_register(self):
        """Verify Section 3 Declared-Total Population Register contains required columns."""
        with open(CANONICAL_TEMPLATE_PATH, "r", encoding="utf-8") as f:
            content = f.read()

        self.assertIn("Declared-Total Population Register", content)
        expected_headers = [
            "Category",
            "Standard ID",
            "Target Metric / Obligation Count",
            "Verification Mechanism",
            "Public Clause Citation",
        ]
        for header in expected_headers:
            self.assertIn(header, content, f"Missing header '{header}' in Section 3 table")

    def test_canonical_template_table_headers_external_additions(self):
        """Verify Section 4 External Additions & Domain Extensions Registry contains required columns."""
        with open(CANONICAL_TEMPLATE_PATH, "r", encoding="utf-8") as f:
            content = f.read()

        self.assertIn("External Additions & Domain Extensions Registry", content)
        expected_headers = [
            "Extension ID",
            "Category",
            "Standard / Baseline ID",
            "Declared Total",
            "Public Clause Citation",
        ]
        for header in expected_headers:
            self.assertIn(header, content, f"Missing header '{header}' in Section 4 table")


class TestResearchInventoryParser(unittest.TestCase):
    """Test suite verifying parser extraction and declared-total calculations."""

    def test_parser_can_parse(self):
        parser = ResearchInventoryParser()
        self.assertTrue(parser.can_parse("docs/research/RESEARCH_INVENTORY.md"))
        self.assertTrue(parser.can_parse(SAMPLE_VALID_RESEARCH_INVENTORY))
        self.assertFalse(parser.can_parse("docs/features/feat-01.md"))

    def test_parse_valid_document_extracts_standards(self):
        doc = parse_research_inventory(SAMPLE_VALID_RESEARCH_INVENTORY)
        self.assertIsInstance(doc, ResearchInventoryDocument)
        self.assertEqual(len(doc.standards), 4)

        std0 = doc.standards[0]
        self.assertEqual(std0.standard_id, "ISO/IEC/IEEE 29148:2018")
        self.assertEqual(std0.issuing_body, "ISO/IEC/IEEE")
        self.assertEqual(std0.title, "Systems and Software Engineering — Requirements Engineering")
        self.assertEqual(std0.applicable_clauses, "§6.4.2 ConOps, §6.4.3 OpsCon, §8.4 System Requirements")
        self.assertEqual(std0.obligation_category, "Requirements Engineering")
        self.assertEqual(std0.declared_total, 3)
        self.assertEqual(std0.clause_citation, "ISO/IEC/IEEE 29148:2018 §6.4.2, §6.4.3, §8.4")

    def test_parse_valid_document_extracts_population_register(self):
        doc = parse_research_inventory(SAMPLE_VALID_RESEARCH_INVENTORY)
        self.assertEqual(len(doc.population_register), 5)

        pop0 = doc.population_register[0]
        self.assertEqual(pop0.obligation_id, "OBL-01")
        self.assertEqual(pop0.category, "Normative Obligation")
        self.assertEqual(pop0.standard_id, "ISO/IEC/IEEE 29148:2018")
        self.assertEqual(pop0.obligation_count, 2)
        self.assertEqual(pop0.verification_mechanism, "Inspection & Traceability Audit")
        self.assertEqual(pop0.clause_citation, "ISO/IEC/IEEE 29148:2018 §6.4.2")

    def test_parse_valid_document_extracts_external_additions(self):
        doc = parse_research_inventory(SAMPLE_VALID_RESEARCH_INVENTORY)
        self.assertEqual(len(doc.external_additions), 2)

        ext0 = doc.external_additions[0]
        self.assertEqual(ext0.extension_id, "EXT-01")
        self.assertEqual(ext0.category, "Domain Extension")
        self.assertEqual(ext0.standard_id, "ASTM F3269-17")
        self.assertEqual(ext0.declared_total, 1)
        self.assertEqual(ext0.clause_citation, "ASTM F3269-17 §5.2")

    def test_declared_totals_calculations(self):
        doc = parse_research_inventory(SAMPLE_VALID_RESEARCH_INVENTORY)
        totals_by_std = doc.get_totals_by_standard()
        self.assertEqual(totals_by_std["ISO/IEC/IEEE 29148:2018"], 3)
        self.assertEqual(totals_by_std["NATO STANAG 4586"], 2)
        self.assertEqual(totals_by_std["RTCA DO-178C / DO-254"], 2)
        self.assertEqual(totals_by_std["MIL-STD-882E"], 2)

        self.assertEqual(doc.get_total_declared_obligations(), 9)

        totals_by_cat = doc.get_totals_by_category()
        self.assertEqual(totals_by_cat["Requirements Engineering"], 3)
        self.assertEqual(totals_by_cat["Interoperability"], 2)


class TestClauseCitationValidator(unittest.TestCase):
    """Test suite verifying public clause citation assertions and schema enforcement."""

    def test_valid_clause_citations(self):
        valid_citations = [
            "ISO/IEC/IEEE 29148:2018 §6.4.2",
            "DO-178C §6.3",
            "DO-254 §11.0",
            "MIL-STD-882E Task 201",
            "STANAG 4586 Ed. 4 §3.2, §4.1",
            "JARUS SORA v2.5 Annex B",
            "SAE ARP4754A §5.0",
            "ASTM F3269-17 Section 5.2",
            "RTCA DO-365B Clause 2.2.4",
            "ARP4761 Appendix L",
        ]
        for citation in valid_citations:
            self.assertTrue(
                is_valid_public_clause_citation(citation),
                f"Expected '{citation}' to be valid",
            )

    def test_invalid_clause_citations(self):
        invalid_citations = [
            "",
            "   ",
            "TBD",
            "N/A",
            "None",
            "unknown",
            "internal note",
            "Custom addition without reference",
            "ISO/IEC/IEEE 29148:2018",  # Missing clause locator
            "DO-178C",                 # Missing clause locator
        ]
        for citation in invalid_citations:
            self.assertFalse(
                is_valid_public_clause_citation(citation),
                f"Expected '{citation}' to be invalid",
            )

    def test_validator_accepts_conforming_document(self):
        validator = ResearchInventoryValidator()
        doc = parse_research_inventory(SAMPLE_VALID_RESEARCH_INVENTORY)
        findings = validator.validate_document(doc, "docs/research/RESEARCH_INVENTORY.md")
        self.assertEqual(findings, [], f"Expected 0 findings, got: {findings}")

    def test_validator_flags_uncited_standards(self):
        bad_inventory = SAMPLE_VALID_RESEARCH_INVENTORY.replace(
            "ISO/IEC/IEEE 29148:2018 §6.4.2, §6.4.3, §8.4",
            "TBD",
        )
        validator = ResearchInventoryValidator()
        doc = parse_research_inventory(bad_inventory)
        findings = validator.validate_document(doc, "docs/research/RESEARCH_INVENTORY.md")
        self.assertTrue(any("invalid_citation" in f.rule_id or "citation" in str(f).lower() for f in findings))

    def test_validator_flags_uncited_population_entries(self):
        bad_inventory = SAMPLE_VALID_RESEARCH_INVENTORY.replace(
            "ISO/IEC/IEEE 29148:2018 §6.4.2",
            "N/A",
        )
        validator = ResearchInventoryValidator()
        doc = parse_research_inventory(bad_inventory)
        findings = validator.validate_document(doc, "docs/research/RESEARCH_INVENTORY.md")
        self.assertTrue(any("invalid_citation" in f.rule_id or "citation" in str(f).lower() for f in findings))

    def test_validator_flags_uncited_external_additions(self):
        bad_inventory = SAMPLE_VALID_RESEARCH_INVENTORY.replace(
            "ASTM F3269-17 §5.2",
            "None",
        )
        validator = ResearchInventoryValidator()
        doc = parse_research_inventory(bad_inventory)
        findings = validator.validate_document(doc, "docs/research/RESEARCH_INVENTORY.md")
        self.assertTrue(any("invalid_citation" in f.rule_id or "uncited" in str(f).lower() or "citation" in str(f).lower() for f in findings))

    def test_validator_flags_declared_total_mismatch(self):
        bad_inventory = re.sub(
            r'(\|\s*ISO/IEC/IEEE 29148:2018\s*\|.*?\|\s*)3(\s*\|)',
            r'\g<1>99\2',
            SAMPLE_VALID_RESEARCH_INVENTORY,
        )
        validator = ResearchInventoryValidator()
        doc = parse_research_inventory(bad_inventory)
        findings = validator.validate_document(doc, "docs/research/RESEARCH_INVENTORY.md")
        self.assertTrue(any("total_mismatch" in f.rule_id or "total" in str(f).lower() for f in findings))

    def test_parse_and_validate_canonical_template_with_tokens(self):
        """Verify that the canonical template parses cleanly and passes validation with is_template=True."""
        with open(CANONICAL_TEMPLATE_PATH, "r", encoding="utf-8") as f:
            template_content = f.read()

        doc = parse_research_inventory(template_content)
        self.assertGreaterEqual(len(doc.standards), 8)
        self.assertGreaterEqual(len(doc.population_register), 8)
        self.assertGreaterEqual(len(doc.external_additions), 2)
        self.assertGreaterEqual(len(doc.clause_allocations), 8)

        validator = ResearchInventoryValidator()
        findings = validator.validate_document(
            doc,
            "RESEARCH_INVENTORY_CANONICAL_TEMPLATE.md",
            content=template_content,
            is_template=True,
        )
        self.assertEqual(findings, [], f"Expected 0 findings on canonical template, got: {findings}")

    def test_validator_flags_missing_standards_table(self):
        bad_content = """# Research Inventory\n\n## 1. Scope\nScope description.\n"""
        validator = ResearchInventoryValidator()
        doc = parse_research_inventory(bad_content)
        findings = validator.validate_document(doc, "docs/research/RESEARCH_INVENTORY.md", content=bad_content)
        self.assertTrue(any("missing_table" in f.rule_id for f in findings))

    def test_validator_flags_schema_header_mismatch(self):
        bad_table = SAMPLE_VALID_RESEARCH_INVENTORY.replace("Standard / Baseline ID", "Unknown Header ID")
        validator = ResearchInventoryValidator()
        doc = parse_research_inventory(bad_table)
        findings = validator.validate_document(doc, "docs/research/RESEARCH_INVENTORY.md", content=bad_table)
        self.assertTrue(any("schema_mismatch" in f.rule_id for f in findings))

    def test_aggregator_includes_research_inventory_validator(self):
        from parity_auditor.aggregator import AGGREGATING_VALIDATORS
        self.assertIn(ResearchInventoryValidator, AGGREGATING_VALIDATORS)

    def test_workspace_upstream_clean_mode(self):
        """In upstream repository, if docs/research/RESEARCH_INVENTORY.md is not present, validate passes."""
        repo = WorkspaceRepository(PROJECT_ROOT)
        if not repo.is_upstream_compiler_repo():
            self.skipTest("Skipping upstream clean mode test in downstream repository.")
        validator = ResearchInventoryValidator()
        findings = validator.validate(repo)
        self.assertEqual(findings, [])


if __name__ == "__main__":
    unittest.main()


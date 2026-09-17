#!/usr/bin/env python3
# Copyright Gint Atkinson, gint.atkinson@gmail.com
"""
Regression test suite for ISO 29148 Stakeholder and User Class taxonomy (UCL-01..UCL-05).
Covers Micro-Task 1 automated regression test suite.

Verifies:
1. test_user_class_taxonomy_unit_04: 04_USER_CLASSES_AND_STAKEHOLDERS.md and
   04_SYSTEM_CAPABILITIES_AND_FUNCTIONS.md define UCL-01 through UCL-05 and contain
   zero occurrences of legacy UC-01 through UC-05 identifiers.
2. test_user_class_taxonomy_unit_10: 10_MAINTENANCE_AND_GSE_SUPPORT.md references UCL-04
   and contains zero occurrences of legacy UC-04.
3. test_assemble_conops_preserves_ucl: Assembles canonical units via
   scripts.assemble_conops.assemble_conops into a temporary directory and asserts
   UCL-01..UCL-05 and UCL-04 are preserved in the output CONOPS.md and not stripped by regex.
4. test_doc_metadata_validator_architecture: Runs DocMetadataValidator().validate(
   WorkspaceRepository(workspace_dir=REPO_ROOT)) and asserts 0 findings across docs/.
5. test_skill_mermaid_syntax_and_guidance: Verifies skills/spec-conops-engineering/SKILL.md
   contains UCL-xx taxonomy guidance, has no comma in "Energy, Actuation & Safety Tier",
   and wraps "Ground Support Equipment and Staging".
6. test_unit_01_mermaid_subgraphs_and_labels: Verifies SV-1 diagram in
   01_METADATA_AND_OVERVIEW.md has direction TB in every subgraph, no unquoted slashes
   in quoted labels, and all label lines <= 35 characters.
"""

import os
import re
import sys
import tempfile
import unittest
from typing import List

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if REPO_ROOT not in sys.path:
    sys.path.insert(0, REPO_ROOT)

PARITY_SRC = os.path.join(REPO_ROOT, "skills", "spec-orchestrator", "parity_auditor", "src")
if PARITY_SRC not in sys.path:
    sys.path.insert(0, PARITY_SRC)

from parity_auditor.core.workspace import WorkspaceRepository
from parity_auditor.validators.doc_metadata_validator import DocMetadataValidator
from parity_auditor.validators.mermaid_syntax_validator import check_mermaid_text
from scripts.assemble_conops import assemble_conops


def _extract_mermaid_blocks(content: str) -> List[str]:
    """Extract bodies of all ```mermaid code fences from markdown content."""
    blocks: List[str] = []
    in_block = False
    current_block: List[str] = []
    for line in content.splitlines():
        if re.match(r"^\s*```mermaid\s*$", line, re.IGNORECASE):
            in_block = True
            current_block = []
        elif in_block and re.match(r"^\s*```\s*$", line):
            in_block = False
            blocks.append("\n".join(current_block))
        elif in_block:
            current_block.append(line)
    return blocks


class TestUserClassTaxonomy(unittest.TestCase):
    """Regression test suite for User Class taxonomy (UCL-01..UCL-05) and diagram hygiene."""

    def test_user_class_taxonomy_unit_04(self):
        """Verifies 04_USER_CLASSES_AND_STAKEHOLDERS.md and 04_SYSTEM_CAPABILITIES_AND_FUNCTIONS.md
        define UCL-01 through UCL-05 and contain zero occurrences of UC-01 through UC-05."""
        unit_files = [
            os.path.join(
                REPO_ROOT,
                "skills",
                "spec-conops-engineering",
                "resources",
                "units",
                "conops",
                "04_USER_CLASSES_AND_STAKEHOLDERS.md",
            ),
            os.path.join(
                REPO_ROOT,
                "skills",
                "spec-conops-engineering",
                "resources",
                "units",
                "conops",
                "04_SYSTEM_CAPABILITIES_AND_FUNCTIONS.md",
            ),
        ]
        expected_ucls = [f"UCL-0{i}" for i in range(1, 6)]
        forbidden_ucs = [f"UC-0{i}" for i in range(1, 6)]

        for path in unit_files:
            self.assertTrue(os.path.isfile(path), f"File missing: {path}")
            with open(path, "r", encoding="utf-8") as f:
                content = f.read()

            for ucl_id in expected_ucls:
                self.assertIn(
                    ucl_id,
                    content,
                    f"Required user class {ucl_id} not found in {os.path.basename(path)}",
                )

            for uc_id in forbidden_ucs:
                matches = re.findall(rf"\b{re.escape(uc_id)}\b", content)
                self.assertEqual(
                    len(matches),
                    0,
                    f"Found legacy identifier {uc_id} in {os.path.basename(path)}: {matches}",
                )

    def test_user_class_taxonomy_unit_10(self):
        """Verifies 10_MAINTENANCE_AND_GSE_SUPPORT.md references UCL-04 and contains zero occurrences of UC-04."""
        unit_path = os.path.join(
            REPO_ROOT,
            "skills",
            "spec-conops-engineering",
            "resources",
            "units",
            "conops",
            "10_MAINTENANCE_AND_GSE_SUPPORT.md",
        )
        self.assertTrue(os.path.isfile(unit_path), f"File missing: {unit_path}")
        with open(unit_path, "r", encoding="utf-8") as f:
            content = f.read()

        self.assertIn(
            "UCL-04",
            content,
            "Expected UCL-04 reference in 10_MAINTENANCE_AND_GSE_SUPPORT.md",
        )
        matches = re.findall(r"\bUC-04\b", content)
        self.assertEqual(
            len(matches),
            0,
            f"Found legacy identifier UC-04 in 10_MAINTENANCE_AND_GSE_SUPPORT.md: {matches}",
        )

    def test_assemble_conops_preserves_ucl(self):
        """Assembles canonical units via scripts.assemble_conops.assemble_conops into a temp directory
        and asserts UCL-01..UCL-05 and UCL-04 are present in the output CONOPS.md and not stripped by regex."""
        input_dir = os.path.join(REPO_ROOT, "skills", "spec-conops-engineering", "resources", "units")
        with tempfile.TemporaryDirectory() as tmpdir:
            success = assemble_conops(
                input_dir=input_dir,
                output_dir=tmpdir,
                verify_only=False,
            )
            self.assertTrue(success, "assemble_conops() failed for canonical units")
            conops_path = os.path.join(tmpdir, "CONOPS.md")
            self.assertTrue(os.path.isfile(conops_path), "CONOPS.md was not generated in output directory")
            with open(conops_path, "r", encoding="utf-8") as f:
                conops_content = f.read()

            for i in range(1, 6):
                ucl_id = f"UCL-0{i}"
                self.assertIn(
                    ucl_id,
                    conops_content,
                    f"Compiled CONOPS.md is missing required identifier {ucl_id}",
                )
            self.assertIn("UCL-04", conops_content, "Compiled CONOPS.md missing UCL-04")

    def test_doc_metadata_validator_architecture(self):
        """Runs DocMetadataValidator().validate(WorkspaceRepository(workspace_dir=REPO_ROOT)) and asserts 0 findings."""
        repo = WorkspaceRepository(workspace_dir=REPO_ROOT)
        validator = DocMetadataValidator()
        findings = validator.validate(repo)
        self.assertEqual(
            len(findings),
            0,
            f"DocMetadataValidator returned {len(findings)} findings across docs/: {findings}",
        )

    def test_skill_mermaid_syntax_and_guidance(self):
        """Verifies skills/spec-conops-engineering/SKILL.md contains UCL-xx guidance,
        has no comma in 'Energy, Actuation & Safety Tier', and wraps 'Ground Support Equipment and Staging'."""
        skill_path = os.path.join(REPO_ROOT, "skills", "spec-conops-engineering", "SKILL.md")
        self.assertTrue(os.path.isfile(skill_path), f"File missing: {skill_path}")
        with open(skill_path, "r", encoding="utf-8") as f:
            content = f.read()

        # 1. Contains UCL-xx guidance
        self.assertTrue(
            bool(re.search(r"\bUCL-(?:xx|\d{2})\b", content) or "UCL-" in content),
            "skills/spec-conops-engineering/SKILL.md missing UCL-xx taxonomy guidance",
        )

        # 2. Has no comma in "Energy, Actuation & Safety Tier"
        self.assertNotIn(
            "Energy, Actuation & Safety Tier",
            content,
            "Found forbidden comma in 'Energy, Actuation & Safety Tier' in SKILL.md",
        )

        # 3. Wraps "Ground Support Equipment and Staging" (must not exist as unbroken >35 char line)
        self.assertNotIn(
            "Ground Support Equipment and Staging",
            content,
            "Expected 'Ground Support Equipment and Staging' to be line-wrapped with <br/> into lines <= 35 characters",
        )
        self.assertIn(
            "Ground Support Equipment",
            content,
            "Expected 'Ground Support Equipment' to remain present in SKILL.md",
        )

    def test_unit_01_mermaid_subgraphs_and_labels(self):
        """Verifies skills/spec-conops-engineering/resources/units/conops/01_METADATA_AND_OVERVIEW.md
        SV-1 diagram has direction TB in every subgraph, no unquoted slashes in quoted labels,
        and lines <= 35 chars."""
        unit_01_path = os.path.join(
            REPO_ROOT,
            "skills",
            "spec-conops-engineering",
            "resources",
            "units",
            "conops",
            "01_METADATA_AND_OVERVIEW.md",
        )
        self.assertTrue(os.path.isfile(unit_01_path), f"File missing: {unit_01_path}")
        with open(unit_01_path, "r", encoding="utf-8") as f:
            content = f.read()

        blocks = _extract_mermaid_blocks(content)
        self.assertGreaterEqual(len(blocks), 1, "Expected at least one Mermaid diagram in 01_METADATA_AND_OVERVIEW.md")
        sv1_block = blocks[0]

        # 1. Verify direction TB in every subgraph
        subgraph_blocks = re.findall(
            r"subgraph\s+([A-Za-z0-9_]+)(?:\[.*?\])?\s*\n(.*?)(?=\bsubgraph\b|\bend\b)",
            sv1_block,
            re.DOTALL,
        )
        self.assertGreaterEqual(
            len(subgraph_blocks),
            1,
            "Expected subgraphs in SV-1 operational context diagram",
        )
        for sg_name, sg_body in subgraph_blocks:
            has_direction_tb = bool(re.search(r"^\s*direction\s+(?:TB|TD)\b", sg_body, re.MULTILINE))
            self.assertTrue(
                has_direction_tb,
                f"SV-1 subgraph '{sg_name}' is missing mandatory 'direction TB'",
            )

        # 2. Verify no unquoted slashes in quoted labels
        quoted_strings = re.findall(r'"([^"]*)"', sv1_block)
        for s in quoted_strings:
            clean = re.sub(r"</?[a-zA-Z0-9_-]+\s*/?>", "", s)
            self.assertNotIn(
                "/",
                clean,
                f"Found forbidden slash '/' in quoted Mermaid label: \"{s}\"",
            )

        # 3. Verify lines <= 35 chars inside node labels and subgraph titles
        for q_label in quoted_strings:
            lines = q_label.split("<br/>")
            for l in lines:
                clean_line = re.sub(r"</?[a-zA-Z0-9_-]+\s*/?>", "", l).strip()
                self.assertLessEqual(
                    len(clean_line),
                    35,
                    f"Label line exceeds 35 characters ({len(clean_line)} > 35): '{clean_line}'",
                )

        # 4. Verify against standard mermaid validator rules
        findings = check_mermaid_text(sv1_block, source="01_METADATA_AND_OVERVIEW.md")
        rule_ids = [f.rule_id for f in findings]
        self.assertNotIn("mermaid-subgraph-direction-tb-mandated", rule_ids)
        self.assertNotIn("mermaid-quoted-label-slash-forbidden", rule_ids)
        self.assertNotIn("mermaid-node-label-line-wrapping-mandated", rule_ids)
        self.assertNotIn("mermaid-ergonomics-unbroken-label", rule_ids)


if __name__ == "__main__":
    unittest.main()

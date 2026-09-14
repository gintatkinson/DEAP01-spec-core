#!/usr/bin/env python3
# Copyright Gint Atkinson, gint.atkinson@gmail.com
"""
Automated regression tests for MermaidSyntaxValidator directory skipping and ConOps unit 10 compliance (Issue #294).

Realises: [Issue-294/MermaidValidatorDirectorySkipping]
"""

import os
import shutil
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
from parity_auditor.validators.mermaid_syntax_validator import (
    MermaidSyntaxValidator,
    check_mermaid_text,
)


class TestMermaidValidatorDesignSkipping(unittest.TestCase):
    """Realises: [Issue-294/TestMermaidValidatorDesignSkipping]

    Validates that:
    1. MermaidSyntaxValidator skips non-normative documentation directories (docs/audits/, docs/decisions/, docs/designs/).
    2. ConOps unit 10 (10_MAINTENANCE_AND_GSE_SUPPORT.md) passes check_mermaid_text without errors.
    """

    def setUp(self):
        """Create a temporary directory for workspace tests."""
        self.test_dir = tempfile.mkdtemp()

    def tearDown(self):
        """Clean up the temporary directory."""
        shutil.rmtree(self.test_dir, ignore_errors=True)

    def test_mermaid_validator_skips_non_normative_directories(self):
        """Verify that MermaidSyntaxValidator skips docs/designs/, docs/audits/, and docs/decisions/."""
        # Create directories
        docs_dir = os.path.join(self.test_dir, "docs")
        designs_dir = os.path.join(docs_dir, "designs")
        audits_dir = os.path.join(docs_dir, "audits")
        decisions_dir = os.path.join(docs_dir, "decisions")
        features_dir = os.path.join(docs_dir, "features")

        for d in [designs_dir, audits_dir, decisions_dir, features_dir]:
            os.makedirs(d, exist_ok=True)

        bad_diagram = (
            "```mermaid\n"
            "flowchart TD\n"
            '    A["ESAD powered on, safety pin in place"] --> B["Normal Mode"]\n'
            "```\n"
        )

        with open(os.path.join(designs_dir, "design_doc.md"), "w", encoding="utf-8") as f:
            f.write(bad_diagram)

        with open(os.path.join(audits_dir, "audit_doc.md"), "w", encoding="utf-8") as f:
            f.write(bad_diagram)

        with open(os.path.join(decisions_dir, "decision_doc.md"), "w", encoding="utf-8") as f:
            f.write(bad_diagram)

        repo = WorkspaceRepository(self.test_dir)
        validator = MermaidSyntaxValidator()
        findings = validator.validate(repo)

        self.assertEqual(
            findings,
            [],
            f"Expected non-normative docs to be skipped, but got findings: {findings}",
        )

        # Now put bad diagram in features_dir (normative) and assert it is caught
        with open(os.path.join(features_dir, "feat_doc.md"), "w", encoding="utf-8") as f:
            f.write(bad_diagram)

        findings_normative = validator.validate(repo)
        self.assertTrue(
            len(findings_normative) > 0,
            "Expected bad diagram in docs/features to be flagged by validator",
        )
        self.assertTrue(
            any("docs/features/feat_doc.md" in f.location for f in findings_normative),
            f"Expected finding location to be docs/features/feat_doc.md, got: {findings_normative}",
        )

    def test_conops_unit_10_passes_mermaid_validation(self):
        """Verify that 10_MAINTENANCE_AND_GSE_SUPPORT.md passes check_mermaid_text without errors."""
        unit_10_path = os.path.join(
            repo_root,
            "skills",
            "spec-conops-engineering",
            "resources",
            "units",
            "conops",
            "10_MAINTENANCE_AND_GSE_SUPPORT.md",
        )
        self.assertTrue(os.path.exists(unit_10_path), f"File not found: {unit_10_path}")

        with open(unit_10_path, "r", encoding="utf-8") as f:
            content = f.read()

        findings = check_mermaid_text(content, source="10_MAINTENANCE_AND_GSE_SUPPORT.md")
        self.assertEqual(
            findings,
            [],
            f"10_MAINTENANCE_AND_GSE_SUPPORT.md has Mermaid validation errors: {[str(f) for f in findings]}",
        )

        # Also check .agents copy if present
        agents_unit_10 = os.path.join(
            repo_root,
            ".agents",
            "skills",
            "spec-conops-engineering",
            "resources",
            "units",
            "conops",
            "10_MAINTENANCE_AND_GSE_SUPPORT.md",
        )
        if os.path.exists(agents_unit_10):
            with open(agents_unit_10, "r", encoding="utf-8") as f:
                agents_content = f.read()
            agents_findings = check_mermaid_text(agents_content, source=".agents/.../10_MAINTENANCE_AND_GSE_SUPPORT.md")
            self.assertEqual(
                agents_findings,
                [],
                f".agents 10_MAINTENANCE_AND_GSE_SUPPORT.md has Mermaid validation errors: {[str(f) for f in agents_findings]}",
            )


if __name__ == "__main__":
    unittest.main()

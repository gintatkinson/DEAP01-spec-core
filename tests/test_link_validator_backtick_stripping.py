# Copyright Gint Atkinson, gint.atkinson@gmail.com
"""
Unit tests for LinkValidator code span stripping and placeholder broadening (Issue #292).

Verifies:
1. Inline backtick code spans like `[OEM Airframe Spec](schema/extracted/oem_spec.md)` are stripped and ignored.
2. Fenced code blocks with backticks (```) and tildes (~~~) are stripped and ignored.
3. Code styling inside link anchor text (e.g. [`Class`](valid.md)) is preserved and validated.
4. Broadened placeholders ('schema/...', 'model.sysml', 'oem_spec.md', '*_results.md', wildcard paths) are skipped.
5. Real broken markdown links outside code blocks/spans are still detected.
"""

import os
import sys
import tempfile
import unittest

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if REPO_ROOT not in sys.path:
    sys.path.insert(0, REPO_ROOT)

PARITY_SRC = os.path.join(REPO_ROOT, "skills", "spec-orchestrator", "parity_auditor", "src")
if PARITY_SRC not in sys.path:
    sys.path.insert(0, PARITY_SRC)

from parity_auditor.core.workspace import WorkspaceRepository
from parity_auditor.validators.link_validator import LinkValidator


class TestLinkValidatorBacktickStripping(unittest.TestCase):
    def setUp(self):
        self.validator = LinkValidator()

    def test_inline_backtick_code_span_ignored(self):
        """Verify inline code spans containing markdown links are not parsed as live links."""
        with tempfile.TemporaryDirectory() as tmpdir:
            docs_dir = os.path.join(tmpdir, "docs")
            os.makedirs(docs_dir, exist_ok=True)

            doc_path = os.path.join(docs_dir, "test_inline.md")
            with open(doc_path, "w", encoding="utf-8") as f:
                f.write(
                    "# Test Document\n\n"
                    "Here is an example: `[OEM Airframe Spec](schema/extracted/oem_spec.md)` in backticks.\n"
                    "Another example: `` `[Another](schema/test.md)` ``.\n"
                )

            repo = WorkspaceRepository(tmpdir)
            findings = self.validator.validate(repo)
            self.assertEqual(
                len(findings),
                0,
                f"Expected 0 findings for inline backtick links, but got: {findings}",
            )

    def test_fenced_code_blocks_backticks_and_tildes_ignored(self):
        """Verify fenced code blocks (``` and ~~~) are stripped and links inside are ignored."""
        with tempfile.TemporaryDirectory() as tmpdir:
            docs_dir = os.path.join(tmpdir, "docs")
            os.makedirs(docs_dir, exist_ok=True)

            doc_path = os.path.join(docs_dir, "test_fenced.md")
            with open(doc_path, "w", encoding="utf-8") as f:
                f.write(
                    "# Fenced Code Block Document\n\n"
                    "```markdown\n"
                    "[Nonexistent In Backticks](schema/extracted/fake_spec.md)\n"
                    "https://github.com/org/repo/blob/main/fake/path.md\n"
                    "```\n\n"
                    "~~~\n"
                    "[Nonexistent In Tildes](docs/reports/simulink_results/fake_results.md)\n"
                    "~~~\n"
                )

            repo = WorkspaceRepository(tmpdir)
            findings = self.validator.validate(repo)
            self.assertEqual(
                len(findings),
                0,
                f"Expected 0 findings for links inside code fences, but got: {findings}",
            )

    def test_link_anchor_text_with_code_spans_preserved(self):
        """Verify links with backticks inside anchor text (e.g. [`Class`](path)) are preserved and checked."""
        with tempfile.TemporaryDirectory() as tmpdir:
            docs_dir = os.path.join(tmpdir, "docs")
            os.makedirs(docs_dir, exist_ok=True)

            # Create existing target file
            target_path = os.path.join(docs_dir, "existing.md")
            with open(target_path, "w", encoding="utf-8") as f:
                f.write("# Existing Target\n")

            doc_path = os.path.join(docs_dir, "test_anchor.md")
            with open(doc_path, "w", encoding="utf-8") as f:
                f.write(
                    "# Anchor Test\n\n"
                    "Valid link with code in anchor: [`Existing`](existing.md)\n"
                    "Broken link with code in anchor: [`Missing`](missing.md)\n"
                )

            repo = WorkspaceRepository(tmpdir)
            findings = self.validator.validate(repo)
            self.assertEqual(len(findings), 1)
            self.assertEqual(findings[0].rule_id, "markdown-broken-link-reference")
            self.assertIn("missing.md", str(findings[0]))

    def test_broadened_placeholders_and_wildcards(self):
        """Verify placeholder tokens and wildcard paths pass validation without findings."""
        with tempfile.TemporaryDirectory() as tmpdir:
            docs_dir = os.path.join(tmpdir, "docs")
            os.makedirs(docs_dir, exist_ok=True)

            doc_path = os.path.join(docs_dir, "test_placeholders.md")
            with open(doc_path, "w", encoding="utf-8") as f:
                f.write(
                    "# Placeholder Document\n\n"
                    "- [OEM Airframe Spec](schema/.../oem_spec.md)\n"
                    "- [OEM Spec](schema/extracted/oem_spec.md)\n"
                    "- [Model Schema](../../schema/model.sysml)\n"
                    "- [Simulink Results](docs/reports/simulink_results/*_results.md)\n"
                    "- [Example Results](docs/reports/simulink_results/EXAMPLE_results.md)\n"
                    "- [Example Model](../../schema/EXAMPLE_model.sysml)\n"
                    "- [Wildcard Spec](docs/reports/simulink_results/*.md)\n"
                )

            repo = WorkspaceRepository(tmpdir)
            findings = self.validator.validate(repo)
            self.assertEqual(
                len(findings),
                0,
                f"Expected 0 findings for placeholder links, but got: {findings}",
            )

    def test_real_broken_links_still_detected(self):
        """Verify real broken markdown links outside code blocks are still detected."""
        with tempfile.TemporaryDirectory() as tmpdir:
            docs_dir = os.path.join(tmpdir, "docs")
            os.makedirs(docs_dir, exist_ok=True)

            doc_path = os.path.join(docs_dir, "test_broken.md")
            with open(doc_path, "w", encoding="utf-8") as f:
                f.write(
                    "# Document with Broken Link\n\n"
                    "Here is a [Broken Link](docs/definitely_missing_file.md).\n"
                )

            repo = WorkspaceRepository(tmpdir)
            findings = self.validator.validate(repo)
            self.assertEqual(len(findings), 1)
            self.assertEqual(findings[0].rule_id, "markdown-broken-link-reference")
            self.assertIn("docs/definitely_missing_file.md", str(findings[0]))


if __name__ == "__main__":
    unittest.main()

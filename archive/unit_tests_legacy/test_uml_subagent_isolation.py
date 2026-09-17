"""
Unit tests for subagent isolation marker validation in UmlValidator (Issue #215).

Verifies that:
1. CommonMark metadata tables with `| **Generation Mode** | subagent |` pass without error.
2. CommonMark metadata tables with variants (`| Generation Mode | subagent |`, backticks, quotes) pass without error.
3. YAML frontmatter with `generation_mode: subagent` continues to pass for backwards compatibility.
4. Documents without the subagent marker (in both frontmatter and table) are flagged with
   finding 'specification-requires-the-subagent-generation-mode-marker'.
"""

import os
import sys
import unittest

repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if repo_root not in sys.path:
    sys.path.insert(0, repo_root)

parity_src = os.path.join(repo_root, "skills", "spec-orchestrator", "parity_auditor", "src")
if parity_src not in sys.path:
    sys.path.insert(0, parity_src)

from parity_auditor.validators.uml import UmlValidator


class TestUmlSubagentIsolation(unittest.TestCase):
    def setUp(self):
        self.validator = UmlValidator()

    def test_commonmark_metadata_table_with_bold_key_passes(self):
        """Verify CommonMark metadata table with bold key passes."""
        content = """# Feature: Test Feature

| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | 101 |
| **Title** | Test Feature |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/SystemModel.sysml](../../schema/SystemModel.sysml) |

## Description
Feature description.
"""
        errors = []
        self.validator._validate_subagent_isolation(content, "Feature", "feat-01.md", errors)
        self.assertEqual(errors, [])

    def test_commonmark_metadata_table_without_bold_passes(self):
        """Verify CommonMark metadata table without bold key passes."""
        content = """# Epic: Test Epic

| Attribute | Specification Detail |
| :--- | :--- |
| Issue ID | 1 |
| Title | Test Epic |
| Generation Mode | subagent |
| Specification Source | [schema/SystemModel.sysml](../../schema/SystemModel.sysml) |
"""
        errors = []
        self.validator._validate_subagent_isolation(content, "Epic", "epic-01.md", errors)
        self.assertEqual(errors, [])

    def test_commonmark_metadata_table_with_backticks_and_quotes_passes(self):
        """Verify CommonMark metadata table with backticks and quotes passes."""
        content = """# User Story: Test Story

| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | 201 |
| **Generation Mode** | `subagent` |
| **Specification Source** | [schema/SystemModel.sysml](../../schema/SystemModel.sysml) |
"""
        errors = []
        self.validator._validate_subagent_isolation(content, "User Story", "us-01.md", errors)
        self.assertEqual(errors, [])

    def test_commonmark_metadata_table_with_subagent_drafted_passes(self):
        """Verify CommonMark metadata table with Subagent Drafted: true passes."""
        content = """# Use Case: Test Use Case

| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | 301 |
| **Subagent Drafted** | true |
| **Specification Source** | [schema/SystemModel.sysml](../../schema/SystemModel.sysml) |
"""
        errors = []
        self.validator._validate_subagent_isolation(content, "Use Case", "uc-01.md", errors)
        self.assertEqual(errors, [])

    def test_yaml_frontmatter_passes(self):
        """Verify legacy YAML frontmatter with generation_mode: subagent passes."""
        content = """---
generation_mode: subagent
title: "Legacy Feature"
---

# Legacy Feature
"""
        errors = []
        self.validator._validate_subagent_isolation(content, "Feature", "feat-legacy.md", errors)
        self.assertEqual(errors, [])

    def test_missing_subagent_marker_is_flagged(self):
        """Verify missing subagent marker triggers Finding."""
        content = """# Feature: Unmarked Feature

| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | 102 |
| **Title** | Unmarked Feature |
| **Specification Source** | [schema/SystemModel.sysml](../../schema/SystemModel.sysml) |

## Description
Feature without subagent tag.
"""
        errors = []
        self.validator._validate_subagent_isolation(content, "Feature", "feat-unmarked.md", errors)
        self.assertEqual(len(errors), 1)
        finding = errors[0]
        self.assertEqual(finding.rule_id, "specification-requires-the-subagent-generation-mode-marker")
        self.assertEqual(finding.location, "feat-unmarked.md")

    def test_yaml_frontmatter_non_subagent_is_flagged(self):
        """Verify frontmatter without subagent tag is flagged."""
        content = """---
generation_mode: manual
title: "Manual Feature"
---

# Manual Feature
"""
        errors = []
        self.validator._validate_subagent_isolation(content, "Feature", "feat-manual.md", errors)
        self.assertEqual(len(errors), 1)
        self.assertEqual(errors[0].rule_id, "specification-requires-the-subagent-generation-mode-marker")


if __name__ == "__main__":
    unittest.main()

#!/usr/bin/env python3
# Copyright Gint Atkinson, gint.atkinson@gmail.com
"""
Unit tests for Mermaid Syntax Validator (Issue #200).

Validates:
1. Detection of commas inside quoted Mermaid labels in graph, flowchart, and state diagrams.
2. Detection of slashes inside quoted Mermaid labels in graph, flowchart, and state diagrams.
3. Clean compliant labels (using hyphens and spaces) passing with 0 findings.
4. Workspace scanning and Finding object properties.
"""

import os
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
    validate_mermaid_quoted_label_content,
)
from parity_auditor.core.findings import Finding


class TestMermaidSyntaxValidator(unittest.TestCase):
    def test_detects_comma_in_quoted_label_flowchart(self):
        """Verify detection of commas in quoted flowchart/graph node labels."""
        bad_md = """
```mermaid
flowchart TD
    A["ESAD powered on, safety pin in place"] --> B["Normal Mode"]
```
"""
        findings = check_mermaid_text(bad_md, source="bad_flowchart.md")
        rule_ids = [f.rule_id for f in findings]
        self.assertIn("mermaid-quoted-label-comma-forbidden", rule_ids)
        comma_findings = [f for f in findings if f.rule_id == "mermaid-quoted-label-comma-forbidden"]
        self.assertEqual(len(comma_findings), 1)
        self.assertIn("ESAD powered on, safety pin in place", str(comma_findings[0]))
        self.assertIn("GitLab's pinned Glfm renderer rejects this content", str(comma_findings[0]))

    def test_detects_slash_in_quoted_label_graph(self):
        """Verify detection of slashes in quoted graph node labels."""
        bad_md = """
```mermaid
graph TD
    Node["SitaWare HQ / ATAK / DELTA"] --> OutNode["Consumer Hub"]
```
"""
        findings = check_mermaid_text(bad_md, source="bad_graph.md")
        rule_ids = [f.rule_id for f in findings]
        self.assertIn("mermaid-quoted-label-slash-forbidden", rule_ids)
        slash_findings = [f for f in findings if f.rule_id == "mermaid-quoted-label-slash-forbidden"]
        self.assertEqual(len(slash_findings), 1)
        self.assertIn("SitaWare HQ / ATAK / DELTA", str(slash_findings[0]))

    def test_detects_comma_in_quoted_transition_statediagram(self):
        """Verify detection of commas in quoted stateDiagram-v2 transitions."""
        bad_md = """
```mermaid
stateDiagram-v2
    [*] --> Armed
    Armed --> Disabled: "ESAD powered on, safety pin in place"
    Disabled --> [*]
```
"""
        findings = check_mermaid_text(bad_md, source="bad_state.md")
        rule_ids = [f.rule_id for f in findings]
        self.assertIn("mermaid-quoted-label-comma-forbidden", rule_ids)
        comma_findings = [f for f in findings if f.rule_id == "mermaid-quoted-label-comma-forbidden"]
        self.assertEqual(len(comma_findings), 1)
        self.assertIn("ESAD powered on, safety pin in place", str(comma_findings[0]))

    def test_detects_slash_in_quoted_transition_statediagram(self):
        """Verify detection of slashes in quoted stateDiagram transitions."""
        bad_md = """
```mermaid
stateDiagram
    StateA --> StateB: "Action / Event Handler"
```
"""
        findings = check_mermaid_text(bad_md, source="bad_state_slash.md")
        rule_ids = [f.rule_id for f in findings]
        self.assertIn("mermaid-quoted-label-slash-forbidden", rule_ids)
        slash_findings = [f for f in findings if f.rule_id == "mermaid-quoted-label-slash-forbidden"]
        self.assertEqual(len(slash_findings), 1)
        self.assertIn("Action / Event Handler", str(slash_findings[0]))

    def test_detects_both_comma_and_slash_in_same_label(self):
        """Verify that a label containing both comma and slash triggers both findings."""
        bad_md = """
```mermaid
flowchart TD
    A["Primary / Secondary, Auto-Failover"] --> B["Active Node"]
```
"""
        findings = check_mermaid_text(bad_md, source="bad_combo.md")
        rule_ids = [f.rule_id for f in findings]
        self.assertIn("mermaid-quoted-label-comma-forbidden", rule_ids)
        self.assertIn("mermaid-quoted-label-slash-forbidden", rule_ids)

    def test_compliant_labels_pass_cleanly(self):
        """Verify compliant Mermaid diagrams with hyphens and spaces pass with 0 findings."""
        clean_md = """
# Compliant Mermaid Diagrams

```mermaid
flowchart TD
    A["ESAD powered on - safety pin in place"] --> B["Normal Mode"]
    Node["SitaWare HQ - ATAK - DELTA"] --> OutNode["Consumer Hub"]
```

```mermaid
stateDiagram-v2
    [*] --> Armed
    Armed --> Disabled: "ESAD powered on - safety pin in place"
    Disabled --> Ready: "Action - Event Handler"
    Ready --> [*]
```

```mermaid
graph TD
    subgraph "System Boundary"
        X["Component A (Active)"] --> Y["Component B (Standby)"]
    end
```
"""
        findings = check_mermaid_text(clean_md, source="clean_diagrams.md")
        self.assertEqual(len(findings), 0, f"Expected 0 findings but got: {findings}")

    def test_validate_mermaid_quoted_label_content_helper(self):
        """Direct test for helper function validate_mermaid_quoted_label_content."""
        line_with_comma = 'Armed --> Disabled: "ESAD powered on, safety pin in place"'
        res = validate_mermaid_quoted_label_content(line_with_comma)
        self.assertEqual(len(res), 1)
        self.assertEqual(res[0][0], '"ESAD powered on, safety pin in place"')
        self.assertEqual(res[0][1], [','])

        line_with_slash = 'Node["SitaWare HQ / ATAK / DELTA"]'
        res = validate_mermaid_quoted_label_content(line_with_slash)
        self.assertEqual(len(res), 1)
        self.assertEqual(res[0][0], '"SitaWare HQ / ATAK / DELTA"')
        self.assertEqual(res[0][1], ['/'])

        line_clean = 'Node["SitaWare HQ - ATAK - DELTA"]'
        res = validate_mermaid_quoted_label_content(line_clean)
        self.assertEqual(len(res), 0)

    def test_workspace_scanning(self):
        """Verify MermaidSyntaxValidator scans workspace documents correctly."""
        with tempfile.TemporaryDirectory() as tmpdir:
            docs_dir = os.path.join(tmpdir, "docs")
            os.makedirs(docs_dir, exist_ok=True)

            with open(os.path.join(docs_dir, "clean.md"), "w", encoding="utf-8") as f:
                f.write("""# Clean Document
```mermaid
flowchart TD
    A["Clean Label"] --> B["Another Clean Label"]
```
""")

            with open(os.path.join(docs_dir, "defect.md"), "w", encoding="utf-8") as f:
                f.write("""# Defect Document
```mermaid
flowchart TD
    A["Bad, Comma"] --> B["Bad / Slash"]
```
""")

            repo = WorkspaceRepository(tmpdir)
            validator = MermaidSyntaxValidator()
            findings = validator.validate(repo, search_dirs=[docs_dir])

            self.assertEqual(len(findings), 2)
            rule_ids = {f.rule_id for f in findings}
            self.assertIn("mermaid-quoted-label-comma-forbidden", rule_ids)
            self.assertIn("mermaid-quoted-label-slash-forbidden", rule_ids)
            self.assertTrue(all(isinstance(f, Finding) for f in findings))


if __name__ == "__main__":
    unittest.main()

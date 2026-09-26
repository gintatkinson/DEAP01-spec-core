#!/usr/bin/env python3
# Copyright Gint Atkinson, gint.atkinson@gmail.com
"""
Unit and integration tests for Gate 30 (Architecture Viewpoint & Diagram Completeness Validator).

Verifies:
1. Canonical architecture diagrams across 5 viewpoints (DoDAF 2.02 / OMG UAF v2.0 / STPA / SORA).
2. Fail-closed detection of missing architecture corpus in downstream workspaces (Issue #365).
3. Upstream clean landing zone preservation.
4. Mermaid fence integrity, non-empty block verification, and valid diagram types.
5. Fail-closed target_diagrams & allow_partial_snippets (Issue #350).
"""

import os
import shutil
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
from parity_auditor.validators.architecture_viewpoint_validator import (
    ArchitectureViewpointValidator,
    CANONICAL_DIAGRAMS,
    CONOPS_CANONICAL_DIAGRAMS,
    RULE_CORPUS_MISSING,
    RULE_DIAGRAM_MISSING,
    RULE_UNCLOSED_FENCE,
    RULE_EMPTY_DIAGRAM,
    RULE_INCOMPLETE_DIAGRAM,
    RULE_SYNTAX_ERROR,
    validate_architecture_viewpoints,
)

RULE_MISSING_DIAGRAM = RULE_DIAGRAM_MISSING
RULE_INVALID_DIAGRAM = RULE_SYNTAX_ERROR


class TestArchitectureViewpointValidator(unittest.TestCase):
    def setUp(self):
        self.test_dir = tempfile.mkdtemp()
        os.makedirs(os.path.join(self.test_dir, "docs", "conops"))
        os.makedirs(os.path.join(self.test_dir, "docs", "interfaces"))
        os.makedirs(os.path.join(self.test_dir, "docs", "safety"))
        self.repo = WorkspaceRepository(self.test_dir)
        self.validator = ArchitectureViewpointValidator()

    def tearDown(self):
        shutil.rmtree(self.test_dir)

    def _write_file(self, rel_path, content):
        full_path = os.path.join(self.test_dir, rel_path)
        os.makedirs(os.path.dirname(full_path), exist_ok=True)
        with open(full_path, "w", encoding="utf-8") as f:
            f.write(content)

    def test_canonical_diagrams_list_count(self):
        expected_diagrams = [
            "OV-1", "OV-2", "SV-1", "SV-2", "ICD-N2", "OV-5b", "OV-6c", "SV-10b",
            "STPA-CONTROL", "STPA-RTA", "SPATIAL-4D"
        ]
        for diag in expected_diagrams:
            self.assertIn(diag, CANONICAL_DIAGRAMS)
        self.assertTrue(len(CANONICAL_DIAGRAMS) >= 11)

    def test_downstream_missing_architecture_corpus_fails_closed(self):
        """Issue #365: downstream workspace with 0 architecture specs fails closed with RULE_CORPUS_MISSING."""
        with tempfile.TemporaryDirectory() as empty_dir:
            repo = WorkspaceRepository(empty_dir)
            findings = self.validator.validate(repo, allow_missing_specs=True)
            self.assertGreater(len(findings), 0, "Gate 30 must fail closed on missing architecture corpus")
            self.assertEqual(findings[0].rule_id, RULE_CORPUS_MISSING)

    def test_upstream_clean_landing_zones_pass(self):
        """Upstream compiler repository with clean landing zones returns empty findings."""
        with tempfile.TemporaryDirectory() as upstream_dir:
            os.makedirs(os.path.join(upstream_dir, ".pipeline", "upstream"), exist_ok=True)
            repo = WorkspaceRepository(upstream_dir)
            findings = self.validator.validate(repo, allow_missing_specs=False)
            self.assertEqual(len(findings), 0, "Upstream clean landing zone must return 0 findings")

    def test_all_diagrams_present_and_valid(self):
        conops = """
# System Context
```mermaid
graph TB
    %% OV-1
    A[System] --> B[External]
```

# Operational Node Connectivity
```mermaid
graph TB
    %% OV-2
    A[Node 1] --> B[Node 2]
```

# Super-System Segment Allocation
```mermaid
graph TB
    %% SV-1
    subgraph segment1
        A1 --> A2
    end
    subgraph segment2
        B1 --> B2
    end
    segment1 ~~~ segment2
```

# Subsystem Interconnect
```mermaid
graph TB
    %% SV-2
    A[Subsystem] --> B[Bus]
```

# Operational Activity Functional Flow
```mermaid
graph TB
    %% OV-5b
    A[OA-1] --> B[OA-2]
```

# Scenario Lifeline Sequences
```mermaid
sequenceDiagram
    %% OV-6c
    A->>B: Message
    B->>A: Reply
```

# Operational Activity Decomposition Tree
```mermaid
graph TB
    %% OV-5a
    Mission --> OA1[OA-1]
    Mission --> OA2[OA-2]
    OA1 --> OA1a[OA-1a]
    OA1 --> OA1b[OA-1b]
```

# Organizational Command
```mermaid
graph TB
    %% OV-4
    Commander --> Director
    Director --> Supervisor
    Supervisor --> Operator
```

# Function-to-Subsystem
```mermaid
graph TB
    %% SV-4
    subgraph Subsystem1
        FN-1 --> FN-2
        FN-2 --> FN-3
    end
```

# Master System Lifecycle State Machine
```mermaid
stateDiagram-v2
    %% SV-10b
    [*] --> Init
    Init --> Standby
    Standby --> Nominal
```

# 4D Spatial Volume
```mermaid
graph TB
    %% SPATIAL-4D
    A[Flight Geography] --> B[Operational Volume]
    B --> C[Containment Buffer]
```
"""
        self._write_file("docs/conops/CONOPS.md", conops)

        icd = """
# N^2 Physical Interface Matrix
```mermaid
graph TB
    %% ICD-N2
    A[Subsystem 1] --> B[Subsystem 2]
```
"""
        self._write_file("docs/interfaces/ICD_01_SYSTEM_INTERFACE_MATRIX.md", icd)

        stpa = """
# Hierarchical Safety Control Structure
```mermaid
graph TB
    %% STPA-CONTROL
    A[Controller] --> B[Actuator]
    C[Sensor] --> A
    B --> D[Process]
```

# Run-Time Assurance RTA Statechart
```mermaid
stateDiagram-v2
    %% STPA-RTA
    [*] --> Nominal
    Nominal --> Intervention : trigger
```
"""
        self._write_file("docs/safety/STPA_MATRIX.md", stpa)

        self.repo = WorkspaceRepository(self.test_dir)
        findings = self.validator.validate(self.repo)
        self.assertEqual(len(findings), 0, f"Expected 0 findings, got: {[f.message for f in findings]}")

    def test_missing_diagram_fails_closed(self):
        self._write_file("docs/conops/CONOPS.md", "# CONOPS Document\n\nSome text but no diagrams.\n")
        self._write_file("docs/interfaces/ICD_01_SYSTEM_INTERFACE_MATRIX.md", "# ICD Document\n")
        self._write_file("docs/safety/STPA_MATRIX.md", "# STPA Document\n")

        self.repo = WorkspaceRepository(self.test_dir)
        findings = self.validator.validate(self.repo)
        rule_ids = [f.rule_id for f in findings]

        self.assertIn(RULE_MISSING_DIAGRAM, rule_ids)
        messages = [f.message for f in findings if f.rule_id == RULE_MISSING_DIAGRAM]
        self.assertTrue(any("OV-1" in m for m in messages))
        self.assertTrue(any("SV-2" in m for m in messages))

    def test_unclosed_mermaid_fence_fails(self):
        content = """
# System Context
```mermaid
graph TB
    %% OV-1
    A --> B
"""
        self._write_file("docs/conops/CONOPS.md", content)
        self._write_file("docs/interfaces/ICD_01_SYSTEM_INTERFACE_MATRIX.md", "")
        self._write_file("docs/safety/STPA_MATRIX.md", "")
        self.repo = WorkspaceRepository(self.test_dir)
        findings = self.validator.validate(self.repo)

        rule_ids = [f.rule_id for f in findings]
        self.assertIn(RULE_UNCLOSED_FENCE, rule_ids)

    def test_empty_mermaid_block_fails(self):
        content = """
# System Context
```mermaid

```
"""
        self._write_file("docs/conops/CONOPS.md", content)
        self._write_file("docs/interfaces/ICD_01_SYSTEM_INTERFACE_MATRIX.md", "")
        self._write_file("docs/safety/STPA_MATRIX.md", "")
        self.repo = WorkspaceRepository(self.test_dir)
        findings = self.validator.validate(self.repo)

        rule_ids = [f.rule_id for f in findings]
        self.assertIn(RULE_EMPTY_DIAGRAM, rule_ids)

    def test_diagram_type_enforcement(self):
        content = """
# System Context
```mermaid
sequenceDiagram
    %% OV-1
    A->>B: wrong type for structural view
```
"""
        self._write_file("docs/conops/CONOPS.md", content)
        self._write_file("docs/interfaces/ICD_01_SYSTEM_INTERFACE_MATRIX.md", "")
        self._write_file("docs/safety/STPA_MATRIX.md", "")
        self.repo = WorkspaceRepository(self.test_dir)
        findings = self.validator.validate(self.repo)

        messages = [f.message for f in findings if f.rule_id == RULE_INCOMPLETE_DIAGRAM]
        self.assertTrue(any("OV-1" in m and "invalid" in m for m in messages))

    def test_standalone_validate_function(self):
        self._write_file("docs/conops/CONOPS.md", "# Doc\n")
        findings1 = self.validator.validate(self.repo)
        findings2 = validate_architecture_viewpoints(self.repo)
        self.assertEqual([f.rule_id for f in findings1], [f.rule_id for f in findings2])

    def test_target_diagrams_without_allow_partial_snippets_raises_value_error(self):
        """Verify target_diagrams scoping without allow_partial_snippets=True raises ValueError (Issue #350)."""
        content = """
# Super-System Segment Allocation
```mermaid
graph TB
    %% SV-1
    subgraph segment1
        A1 --> A2
    end
```
"""
        with self.assertRaises(ValueError) as ctx:
            self.validator.validate_markdown_content(
                content, "docs/conops/CONOPS.md", target_diagrams={"SV-1"}
            )
        self.assertIn(
            "target_diagrams scoping is only permitted when allow_partial_snippets=True for isolated snippet unit tests.",
            str(ctx.exception),
        )

        with self.assertRaises(ValueError) as ctx2:
            self.validator.validate_markdown_content(
                content, "docs/conops/CONOPS.md", target_diagrams={"SV-1"}, allow_partial_snippets=False
            )
        self.assertIn(
            "target_diagrams scoping is only permitted when allow_partial_snippets=True for isolated snippet unit tests.",
            str(ctx2.exception),
        )

    def test_target_diagrams_with_allow_partial_snippets_allowed(self):
        """Verify target_diagrams scoping succeeds when allow_partial_snippets=True (Issue #350)."""
        content = """
# Super-System Segment Allocation
```mermaid
graph TB
    %% SV-1
    subgraph segment1
        A1 --> A2
    end
    subgraph segment2
        B1 --> B2
    end
    segment1 ~~~ segment2
```
"""
        findings = self.validator.validate_markdown_content(
            content, "docs/conops/CONOPS.md", target_diagrams={"SV-1"}, allow_partial_snippets=True
        )
        self.assertEqual(findings, [])

    def test_unblinded_full_document_validation_cannot_be_blinded(self):
        """Verify full document validation (target_diagrams=None, allow_partial_snippets=False) asserts all canonical diagrams."""
        content = """
# Single Viewpoint Document
```mermaid
graph TB
    %% SV-1
    subgraph segment1
        A1 --> A2
    end
    subgraph segment2
        B1 --> B2
    end
    segment1 ~~~ segment2
```
"""
        findings = self.validator.validate_markdown_content(
            content, "docs/conops/CONOPS.md", target_diagrams=None, allow_partial_snippets=False
        )
        missing_ids = {
            f.detail.get("diagram_id") for f in findings if f.rule_id == RULE_DIAGRAM_MISSING
        }
        self.assertIn("OV-1", missing_ids)
        self.assertIn("OV-2", missing_ids)
        self.assertIn("SPATIAL-4D", missing_ids)


if __name__ == "__main__":
    unittest.main()

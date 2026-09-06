#!/usr/bin/env python3
# Copyright Gint Atkinson, gint.atkinson@gmail.com
"""
Unit tests for AST Manifest Injection in Subagent Dispatch Contracts (fixes #239).

Verifies:
1. skills/spec-orchestrator/SKILL.md mandates AST Manifest Injection in Item-Level Subagent Context Isolation.
2. skills/spec-orchestrator/SKILL.md Phase 0.5 triggers with AST part def manifest and 4 Universal Failure Dimensions.
3. skills/spec-orchestrator/SKILL.md Phase 0.75 triggers with AST state def prefix families (>= 2 states) and part def nodes.
4. skills/spec-orchestrator/SKILL.md Phases 1, 1.5, 2, and 3 dispatch contracts mandate explicit AST slices.
5. Symmetrical reinforcement across spec-conops-engineering, schema-specification-engineering,
   spec-user-story-engineering, and spec-usecase-engineering SKILL.md files.
"""

import os
import sys
import unittest

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if REPO_ROOT not in sys.path:
    sys.path.insert(0, REPO_ROOT)

ORCHESTRATOR_SKILL_PATH = os.path.join(REPO_ROOT, "skills", "spec-orchestrator", "SKILL.md")
CONOPS_SKILL_PATH = os.path.join(REPO_ROOT, "skills", "spec-conops-engineering", "SKILL.md")
SCHEMA_SKILL_PATH = os.path.join(REPO_ROOT, "skills", "schema-specification-engineering", "SKILL.md")
USER_STORY_SKILL_PATH = os.path.join(REPO_ROOT, "skills", "spec-user-story-engineering", "SKILL.md")
USECASE_SKILL_PATH = os.path.join(REPO_ROOT, "skills", "spec-usecase-engineering", "SKILL.md")


class TestASTManifestDispatchContracts(unittest.TestCase):
    """Test suite asserting AST manifest injection contracts in subagent dispatch specifications."""

    def test_orchestrator_item_level_subagent_dispatch_ast_manifest(self):
        """Verify Item-Level Subagent Context Isolation mandates AST Manifest Injection."""
        with open(ORCHESTRATOR_SKILL_PATH, "r", encoding="utf-8") as f:
            content = f.read()

        self.assertIn("Mandatory AST Manifest Injection", content)
        self.assertIn("For Epics/Features: The exact AST package and part def / item def definition", content)
        self.assertIn("For User Stories: The exact AST action def, port def, and complete state def state machine family", content)
        self.assertIn("For Use Cases: The exact AST use case def with subject part def, actor port contracts", content)

    def test_orchestrator_phase_0_5_trigger_ast_part_defs_and_failure_dimensions(self):
        """Verify Phase 0.5 triggers with AST part def manifest and 4 Universal Failure Dimensions."""
        with open(ORCHESTRATOR_SKILL_PATH, "r", encoding="utf-8") as f:
            content = f.read()

        self.assertIn("Phase 0.5: Normative-Completeness Research Step", content)
        pos_phase_0_5 = content.find("## Phase 0.5:")
        pos_phase_0_75 = content.find("## Phase 0.75:")
        phase_0_5_block = content[pos_phase_0_5:pos_phase_0_75]

        self.assertIn(".pipeline/schema.sysml", phase_0_5_block)
        self.assertIn("AST `part def` component manifest", phase_0_5_block)
        self.assertIn("4 Universal Failure Dimensions (Interface, State, Action, Resource)", phase_0_5_block)

    def test_orchestrator_phase_0_75_trigger_ast_state_defs_and_part_defs(self):
        """Verify Phase 0.75 triggers with AST state def prefix families and part def nodes."""
        with open(ORCHESTRATOR_SKILL_PATH, "r", encoding="utf-8") as f:
            content = f.read()

        pos_phase_0_75 = content.find("## Phase 0.75:")
        pos_phase_1 = content.find("## Phase 1:")
        phase_0_75_block = content[pos_phase_0_75:pos_phase_1]

        self.assertIn("manifest of all AST `state def` prefix families", phase_0_75_block)
        self.assertIn("part def` nodes", phase_0_75_block)

    def test_orchestrator_phases_1_to_3_ast_manifest_slices(self):
        """Verify Phases 1, 1.5, 2, and 3 dispatch contracts mandate AST manifest slices."""
        with open(ORCHESTRATOR_SKILL_PATH, "r", encoding="utf-8") as f:
            content = f.read()

        # Phase 1: Structural Spec Worker
        self.assertIn("extracted AST `part def` / `item def` component manifest", content)
        self.assertIn("injecting the mandatory AST package, part def, port, action, and constraint manifest slices", content)

        # Phase 1.5: Interface Spec Worker
        self.assertIn("extracted AST `port def`, `connection`, `interface def`, and `item flow` manifest slices", content)

        # Phase 2: Behavioral Spec Worker
        self.assertIn("extracted AST `action def`, `port def`, and complete `state def` state machine family slices", content)
        self.assertIn("injecting the mandatory AST action, port, and state machine manifest slices", content)

        # Phase 3: System Interaction Spec Worker
        self.assertIn("extracted AST `use case def`, subject `part def`, and actor port contracts manifest slices", content)
        self.assertIn("injecting the mandatory AST use case def and subject part def manifest slices", content)

    def test_conops_skill_ast_manifest_reinforcement(self):
        """Verify spec-conops-engineering SKILL.md reinforces AST manifest ingestion."""
        with open(CONOPS_SKILL_PATH, "r", encoding="utf-8") as f:
            content = f.read()

        self.assertIn("Mandatory AST Manifest Ingestion", content)
        self.assertIn("AST `state def` prefix families with 2 or more states", content)
        self.assertIn("AST `part def` nodes", content)
        self.assertIn("docs/research/FAILURE_MODE_REGISTRY.md", content)

    def test_schema_specification_engineering_skill_ast_manifest_reinforcement(self):
        """Verify schema-specification-engineering SKILL.md reinforces AST manifest slices for Epics and Features."""
        with open(SCHEMA_SKILL_PATH, "r", encoding="utf-8") as f:
            content = f.read()

        self.assertIn("Mandatory AST Manifest Slice", content)
        self.assertIn("package`, `part def` components, and `capability def` declarations", content)
        self.assertIn("exact AST package and `part def` / `item def` definition with all owned properties, ports, operations, and constraints", content)

    def test_user_story_and_usecase_skills_ast_manifest_reinforcement(self):
        """Verify spec-user-story-engineering and spec-usecase-engineering reinforce AST manifest slices."""
        with open(USER_STORY_SKILL_PATH, "r", encoding="utf-8") as f:
            us_content = f.read()
        with open(USECASE_SKILL_PATH, "r", encoding="utf-8") as f:
            uc_content = f.read()

        self.assertIn("Mandatory AST Manifest Slice", us_content)
        self.assertIn("exact AST `action def`, `port def`, and complete `state def` state machine family", us_content)

        self.assertIn("Mandatory AST Manifest Slice", uc_content)
        self.assertIn("exact AST `use case def` with subject `part def`, actor port contracts", uc_content)


if __name__ == "__main__":
    unittest.main()

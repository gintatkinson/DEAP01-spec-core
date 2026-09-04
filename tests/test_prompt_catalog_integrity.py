#!/usr/bin/env python3
"""
Unit tests for Operator Prompt Catalog Integrity & Domain-Agnostic Synthesis (Issue #198).
Verifies that README.md Section 8.3 and scripts/install_pipeline.sh Section 4.2 contain
100% schema-driven, domain-agnostic prompts conforming to DEAP subagent dispatch rules.
"""

import os
import re
import sys
import unittest

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if REPO_ROOT not in sys.path:
    sys.path.insert(0, REPO_ROOT)

from scripts.lint_subagent_prompt import (
    check_step1_skill_directive,
    check_repository_classification,
    check_leading_code_steering,
)


def extract_prompt_blocks(markdown_content: str, section_header_pattern: str) -> dict:
    """Extract prompt text blocks for Worker 0A, Worker 0B, Worker 0C under the specified section."""
    prompts = {}
    
    # Extract Worker 0A prompt
    match_0a = re.search(
        r"####\s+(?:8\.3\.1|4\.2\.1)\s+Worker 0A[^\n]*\n+```text\n(.*?)```",
        markdown_content,
        re.DOTALL,
    )
    if match_0a:
        prompts["worker_0a"] = match_0a.group(1).strip()

    # Extract Worker 0B prompt
    match_0b = re.search(
        r"####\s+(?:8\.3\.2|4\.2\.2)\s+Worker 0B[^\n]*\n+```text\n(.*?)```",
        markdown_content,
        re.DOTALL,
    )
    if match_0b:
        prompts["worker_0b"] = match_0b.group(1).strip()

    # Extract Worker 0C prompt
    match_0c = re.search(
        r"####\s+(?:8\.3\.3|4\.2\.3)\s+Worker 0C[^\n]*\n+```text\n(.*?)```",
        markdown_content,
        re.DOTALL,
    )
    if match_0c:
        prompts["worker_0c"] = match_0c.group(1).strip()

    return prompts


class TestPromptCatalogIntegrity(unittest.TestCase):
    """Test suite verifying schema-driven, domain-agnostic operator prompt catalog in README and installer."""

    @classmethod
    def setUpClass(cls):
        readme_path = os.path.join(REPO_ROOT, "README.md")
        installer_path = os.path.join(REPO_ROOT, "scripts", "install_pipeline.sh")

        with open(readme_path, "r", encoding="utf-8") as f:
            cls.readme_content = f.read()

        with open(installer_path, "r", encoding="utf-8") as f:
            cls.installer_content = f.read()

        cls.readme_prompts = extract_prompt_blocks(cls.readme_content, r"### 8\.3")
        cls.installer_prompts = extract_prompt_blocks(cls.installer_content, r"### 4\.2")

    def test_all_prompts_extracted(self):
        """Verify that all three worker prompts are successfully parsed from both README and installer."""
        self.assertIn("worker_0a", self.readme_prompts, "Worker 0A prompt missing from README.md Section 8.3")
        self.assertIn("worker_0b", self.readme_prompts, "Worker 0B prompt missing from README.md Section 8.3")
        self.assertIn("worker_0c", self.readme_prompts, "Worker 0C prompt missing from README.md Section 8.3")

        self.assertIn("worker_0a", self.installer_prompts, "Worker 0A prompt missing from install_pipeline.sh Section 4.2")
        self.assertIn("worker_0b", self.installer_prompts, "Worker 0B prompt missing from install_pipeline.sh Section 4.2")
        self.assertIn("worker_0c", self.installer_prompts, "Worker 0C prompt missing from install_pipeline.sh Section 4.2")

    def test_worker_0a_prompt_compliance(self):
        """Verify Worker 0A prompt invariants in README.md and install_pipeline.sh."""
        for source, prompts in [("README.md", self.readme_prompts), ("install_pipeline.sh", self.installer_prompts)]:
            prompt = prompts["worker_0a"]
            
            # Step 1 skill directive on spec-conops-engineering
            self.assertTrue(
                check_step1_skill_directive(prompt),
                f"Worker 0A in {source} failed check_step1_skill_directive",
            )
            self.assertIn(
                "skills/spec-conops-engineering/SKILL.md",
                prompt,
                f"Worker 0A in {source} missing skills/spec-conops-engineering/SKILL.md path",
            )

            # Repository classification
            self.assertTrue(
                check_repository_classification(prompt),
                f"Worker 0A in {source} failed check_repository_classification",
            )
            self.assertIn("DOWNSTREAM_CUSTOMER_PROJECT", prompt)

            # Clean leading steering
            self.assertFalse(
                check_leading_code_steering(prompt),
                f"Worker 0A in {source} has leading code steering",
            )

            # Domain-agnostic operational envelope
            self.assertIn("Schema-derived operational envelope", prompt)
            self.assertIn("physical boundaries, operating dynamics, environmental constraints", prompt)
            self.assertIn("Initialization, Normal Operation, Degraded/Contingency Modes, and Safe Shutdown/Transition", prompt)
            self.assertIn("Dynamic stakeholder roles derived from the system operational context", prompt)

            # Zero hardcoded UAS flight specifics in Worker 0A prompt
            self.assertNotIn("flight altitude boundaries", prompt)
            self.assertNotIn("BVLOS vs VLOS", prompt)
            self.assertNotIn("Remote Pilot in Command", prompt)
            self.assertNotIn("Fail-Safe Contingency RTL", prompt)

            # PROCEED token
            self.assertTrue(prompt.strip().endswith("PROCEED"), f"Worker 0A in {source} missing PROCEED token")

    def test_worker_0b_prompt_compliance(self):
        """Verify Worker 0B prompt invariants in README.md and install_pipeline.sh."""
        for source, prompts in [("README.md", self.readme_prompts), ("install_pipeline.sh", self.installer_prompts)]:
            prompt = prompts["worker_0b"]

            # Step 1 skill directive on spec-orchestrator
            self.assertTrue(
                check_step1_skill_directive(prompt),
                f"Worker 0B in {source} failed check_step1_skill_directive",
            )
            self.assertIn(
                "skills/spec-orchestrator/SKILL.md",
                prompt,
                f"Worker 0B in {source} missing skills/spec-orchestrator/SKILL.md path",
            )

            # Repository classification
            self.assertTrue(
                check_repository_classification(prompt),
                f"Worker 0B in {source} failed check_repository_classification",
            )
            self.assertIn("DOWNSTREAM_CUSTOMER_PROJECT", prompt)

            # Dynamic domain safety framework selection across multi-domains
            self.assertIn("ISO 14971/IEC 62304 for Medical", prompt)
            self.assertIn("EN 50128 for Rail", prompt)
            self.assertIn("DNV-GL for Marine", prompt)
            self.assertIn("ECSS for Space", prompt)
            self.assertIn("ISO 3691-4 for Industrial AGV", prompt)
            self.assertIn("SORA/DO-178C for Aviation", prompt)

            # 8-pillar schema
            self.assertIn("8-pillar schema", prompt)
            self.assertIn("System Losses", prompt)
            self.assertIn("System Hazards", prompt)
            self.assertIn("Hierarchical Control Structure Topology", prompt)
            self.assertIn("Unsafe Control Actions", prompt)
            self.assertIn("Loss Scenarios", prompt)
            self.assertIn("Formal Safety Constraints", prompt)
            self.assertIn("FMECA Criticality Matrix", prompt)

            # PROCEED token
            self.assertTrue(prompt.strip().endswith("PROCEED"), f"Worker 0B in {source} missing PROCEED token")

    def test_worker_0c_prompt_compliance(self):
        """Verify Worker 0C prompt invariants in README.md and install_pipeline.sh."""
        for source, prompts in [("README.md", self.readme_prompts), ("install_pipeline.sh", self.installer_prompts)]:
            prompt = prompts["worker_0c"]

            # Step 1 skill directive on spec-orchestrator
            self.assertTrue(
                check_step1_skill_directive(prompt),
                f"Worker 0C in {source} failed check_step1_skill_directive",
            )
            self.assertIn(
                "skills/spec-orchestrator/SKILL.md",
                prompt,
                f"Worker 0C in {source} missing skills/spec-orchestrator/SKILL.md path",
            )

            # Repository classification
            self.assertTrue(
                check_repository_classification(prompt),
                f"Worker 0C in {source} failed check_repository_classification",
            )
            self.assertIn("DOWNSTREAM_CUSTOMER_PROJECT", prompt)

            # Canonical model generation
            self.assertIn("canonical `DEAP_MODEL.sysml`", prompt)
            self.assertIn("pipeline0_handoff_contract.json", prompt)

            # PROCEED token
            self.assertTrue(prompt.strip().endswith("PROCEED"), f"Worker 0C in {source} missing PROCEED token")

    def test_readme_installer_prompt_parity(self):
        """Verify 100% prompt body parity between README.md Section 8.3 and install_pipeline.sh Section 4.2."""
        self.assertEqual(
            self.readme_prompts["worker_0a"],
            self.installer_prompts["worker_0a"],
            "Worker 0A prompt differs between README.md and install_pipeline.sh",
        )
        self.assertEqual(
            self.readme_prompts["worker_0b"],
            self.installer_prompts["worker_0b"],
            "Worker 0B prompt differs between README.md and install_pipeline.sh",
        )
        self.assertEqual(
            self.readme_prompts["worker_0c"],
            self.installer_prompts["worker_0c"],
            "Worker 0C prompt differs between README.md and install_pipeline.sh",
        )


if __name__ == "__main__":
    unittest.main()

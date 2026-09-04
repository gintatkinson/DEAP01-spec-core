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
    """Extract prompt text blocks for all workers under the specified section."""
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

    # Extract Worker 1A prompt
    match_1a = re.search(
        r"####\s+(?:9\.2\.1|4\.3\.1)\s+Worker 1A[^\n]*\n+```text\n(.*?)```",
        markdown_content,
        re.DOTALL,
    )
    if match_1a:
        prompts["worker_1a"] = match_1a.group(1).strip()

    # Extract Worker 1B prompt
    match_1b = re.search(
        r"####\s+(?:9\.2\.2|4\.3\.2)\s+Worker 1B[^\n]*\n+```text\n(.*?)```",
        markdown_content,
        re.DOTALL,
    )
    if match_1b:
        prompts["worker_1b"] = match_1b.group(1).strip()

    # Extract Worker 1C prompt
    match_1c = re.search(
        r"####\s+(?:9\.2\.3|4\.3\.3)\s+Worker 1C[^\n]*\n+```text\n(.*?)```",
        markdown_content,
        re.DOTALL,
    )
    if match_1c:
        prompts["worker_1c"] = match_1c.group(1).strip()

    # Extract Worker 1D prompt
    match_1d = re.search(
        r"####\s+(?:9\.2\.4|4\.3\.4)\s+Worker 1D[^\n]*\n+```text\n(.*?)```",
        markdown_content,
        re.DOTALL,
    )
    if match_1d:
        prompts["worker_1d"] = match_1d.group(1).strip()

    # Extract Worker 2 prompt
    match_2 = re.search(
        r"####\s+(?:9\.4\.1|4\.5\.1)\s+Worker 2[^\n]*\n+```text\n(.*?)```",
        markdown_content,
        re.DOTALL,
    )
    if match_2:
        prompts["worker_2"] = match_2.group(1).strip()

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
        """Verify that all worker prompts are successfully parsed from both README and installer."""
        for key in ["worker_0a", "worker_0b", "worker_0c", "worker_1a", "worker_1b", "worker_1c", "worker_1d", "worker_2"]:
            self.assertIn(key, self.readme_prompts, f"{key} prompt missing from README.md")
            self.assertIn(key, self.installer_prompts, f"{key} prompt missing from install_pipeline.sh Section 4")

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

    def test_worker_1a_prompt_compliance(self):
        """Verify Worker 1A prompt invariants in README.md and install_pipeline.sh."""
        for source, prompts in [("README.md", self.readme_prompts), ("install_pipeline.sh", self.installer_prompts)]:
            prompt = prompts["worker_1a"]

            self.assertTrue(
                check_step1_skill_directive(prompt),
                f"Worker 1A in {source} failed check_step1_skill_directive",
            )
            self.assertIn(
                "skills/schema-specification-engineering/SKILL.md",
                prompt,
                f"Worker 1A in {source} missing skills/schema-specification-engineering/SKILL.md path",
            )
            self.assertTrue(
                check_repository_classification(prompt),
                f"Worker 1A in {source} failed check_repository_classification",
            )
            self.assertIn("DOWNSTREAM_CUSTOMER_PROJECT", prompt)
            self.assertFalse(
                check_leading_code_steering(prompt),
                f"Worker 1A in {source} has leading code steering",
            )
            self.assertTrue(prompt.strip().endswith("PROCEED"), f"Worker 1A in {source} missing PROCEED token")

    def test_worker_1b_prompt_compliance(self):
        """Verify Worker 1B prompt invariants in README.md and install_pipeline.sh."""
        for source, prompts in [("README.md", self.readme_prompts), ("install_pipeline.sh", self.installer_prompts)]:
            prompt = prompts["worker_1b"]

            self.assertTrue(
                check_step1_skill_directive(prompt),
                f"Worker 1B in {source} failed check_step1_skill_directive",
            )
            self.assertIn(
                "skills/spec-icd-engineering/SKILL.md",
                prompt,
                f"Worker 1B in {source} missing skills/spec-icd-engineering/SKILL.md path",
            )
            self.assertTrue(
                check_repository_classification(prompt),
                f"Worker 1B in {source} failed check_repository_classification",
            )
            self.assertIn("DOWNSTREAM_CUSTOMER_PROJECT", prompt)
            self.assertFalse(
                check_leading_code_steering(prompt),
                f"Worker 1B in {source} has leading code steering",
            )
            self.assertTrue(prompt.strip().endswith("PROCEED"), f"Worker 1B in {source} missing PROCEED token")

    def test_worker_1c_prompt_compliance(self):
        """Verify Worker 1C prompt invariants in README.md and install_pipeline.sh."""
        for source, prompts in [("README.md", self.readme_prompts), ("install_pipeline.sh", self.installer_prompts)]:
            prompt = prompts["worker_1c"]

            self.assertTrue(
                check_step1_skill_directive(prompt),
                f"Worker 1C in {source} failed check_step1_skill_directive",
            )
            self.assertIn(
                "skills/spec-user-story-engineering/SKILL.md",
                prompt,
                f"Worker 1C in {source} missing skills/spec-user-story-engineering/SKILL.md path",
            )
            self.assertTrue(
                check_repository_classification(prompt),
                f"Worker 1C in {source} failed check_repository_classification",
            )
            self.assertIn("DOWNSTREAM_CUSTOMER_PROJECT", prompt)
            self.assertFalse(
                check_leading_code_steering(prompt),
                f"Worker 1C in {source} has leading code steering",
            )
            self.assertTrue(prompt.strip().endswith("PROCEED"), f"Worker 1C in {source} missing PROCEED token")

    def test_worker_1d_prompt_compliance(self):
        """Verify Worker 1D prompt invariants in README.md and install_pipeline.sh."""
        for source, prompts in [("README.md", self.readme_prompts), ("install_pipeline.sh", self.installer_prompts)]:
            prompt = prompts["worker_1d"]

            self.assertTrue(
                check_step1_skill_directive(prompt),
                f"Worker 1D in {source} failed check_step1_skill_directive",
            )
            self.assertIn(
                "skills/spec-usecase-engineering/SKILL.md",
                prompt,
                f"Worker 1D in {source} missing skills/spec-usecase-engineering/SKILL.md path",
            )
            self.assertTrue(
                check_repository_classification(prompt),
                f"Worker 1D in {source} failed check_repository_classification",
            )
            self.assertIn("DOWNSTREAM_CUSTOMER_PROJECT", prompt)
            self.assertFalse(
                check_leading_code_steering(prompt),
                f"Worker 1D in {source} has leading code steering",
            )
            self.assertTrue(prompt.strip().endswith("PROCEED"), f"Worker 1D in {source} missing PROCEED token")

    def test_worker_2_prompt_compliance(self):
        """Verify Worker 2 prompt invariants in README.md and install_pipeline.sh."""
        for source, prompts in [("README.md", self.readme_prompts), ("install_pipeline.sh", self.installer_prompts)]:
            prompt = prompts["worker_2"]

            self.assertTrue(
                check_step1_skill_directive(prompt),
                f"Worker 2 in {source} failed check_step1_skill_directive",
            )
            self.assertIn(
                "skills/feature-driven-implementation/SKILL.md",
                prompt,
                f"Worker 2 in {source} missing skills/feature-driven-implementation/SKILL.md path",
            )
            self.assertTrue(
                check_repository_classification(prompt),
                f"Worker 2 in {source} failed check_repository_classification",
            )
            self.assertIn("DOWNSTREAM_CUSTOMER_PROJECT", prompt)
            self.assertFalse(
                check_leading_code_steering(prompt),
                f"Worker 2 in {source} has leading code steering",
            )
            self.assertTrue(prompt.strip().endswith("PROCEED"), f"Worker 2 in {source} missing PROCEED token")

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

    def test_downstream_readme_scaffolding_sections(self):
        """Verify all 5 catalog sections are present in install_pipeline.sh downstream README."""
        self.assertIn("### 4.1 Master-Worker Subagent Topology", self.installer_content)
        self.assertIn("### 4.2 Pipeline 0 Execution Prompts", self.installer_content)
        self.assertIn("### 4.3 Pipeline 1 Agile Backlog Projection Prompts", self.installer_content)
        self.assertIn("### 4.4 Multi-Provider Backlog Reconciliation Commands", self.installer_content)
        self.assertIn("### 4.5 Pipeline 2 Autonomous Feature Implementation Prompts", self.installer_content)
        self.assertIn("#### 4.4.1 Option A: GitLab SaaS Reconciliation", self.installer_content)
        self.assertIn("#### 4.4.2 Option B: GitLab Self-Managed / SCIF Air-Gapped Reconciliation", self.installer_content)
        self.assertIn("#### 4.4.3 Option C: GitHub Issues Reconciliation", self.installer_content)
        self.assertIn("#### 4.4.4 Option D: Offline Verification & 23-Gate Parity Lock", self.installer_content)


if __name__ == "__main__":
    unittest.main()


#!/usr/bin/env python3
"""
Unit tests for Subagent Prompt Dispatcher (`scripts/dispatch_subagent.py`).
"""

import os
import sys
import tempfile
import subprocess
import unittest

# Ensure project root is in sys.path
TEST_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.abspath(os.path.join(TEST_DIR, ".."))
if PROJECT_ROOT not in sys.path:
    sys.path.insert(0, PROJECT_ROOT)

import json
import shutil

from scripts.dispatch_subagent import (
    dispatch_subagent,
    generate_subagent_prompt,
    validate_skill_path,
    construct_prompt_template,
    resolve_repository_classification,
)
from scripts.lint_subagent_prompt import lint_prompt_text, lint_subagent_prompt, validate_subagent_preflight


class TestDispatchSubagent(unittest.TestCase):
    """Test suite for subagent prompt generation, validation, and CLI execution."""

    def setUp(self):
        self.skill_feature = os.path.join("skills", "feature-driven-implementation", "SKILL.md")
        self.skill_auditor = os.path.join("skills", "adversarial-code-auditor", "SKILL.md")
        self.target_file = os.path.join("scripts", "dispatch_subagent.py")

    def test_validate_skill_path_valid(self):
        """Verify that existing SKILL.md paths are validated without error."""
        validated = validate_skill_path(self.skill_feature, base_dir=PROJECT_ROOT)
        self.assertEqual(validated, self.skill_feature)

    def test_validate_skill_path_nonexistent(self):
        """Verify that non-existent skill paths raise FileNotFoundError."""
        with self.assertRaises(FileNotFoundError):
            validate_skill_path("skills/nonexistent-skill/SKILL.md", base_dir=PROJECT_ROOT)

    def test_validate_skill_path_not_skill_md(self):
        """Verify that paths not ending in SKILL.md raise ValueError."""
        with self.assertRaises(ValueError):
            validate_skill_path("README.md", base_dir=PROJECT_ROOT)

    def test_validate_skill_path_empty(self):
        """Verify that empty skill paths raise ValueError."""
        with self.assertRaises(ValueError):
            validate_skill_path("", base_dir=PROJECT_ROOT)

    def test_generate_subagent_prompt_success(self):
        """Verify that prompt generation constructs a fully compliant, lint-passing prompt."""
        prompt = generate_subagent_prompt(
            skill=self.skill_feature,
            target=self.target_file,
            role="Feature Implementation Worker",
            subagent_type="code_modifier_worker",
            classification="UPSTREAM_SPEC_CORE_COMPILER",
            base_dir=PROJECT_ROOT,
        )

        # Check required components
        self.assertIn("view_file", prompt)
        self.assertIn("SKILL.md", prompt)
        self.assertIn("UPSTREAM_SPEC_CORE_COMPILER", prompt)
        self.assertIn(self.target_file, prompt)
        self.assertIn("Feature Implementation Worker", prompt)
        self.assertIn("code_modifier_worker", prompt)
        self.assertIn("gh issue create", prompt)
        self.assertIn("glab issue create", prompt)
        self.assertIn("PROCEED", prompt)

        # Assert zero lint violations
        lint_errors = lint_prompt_text(prompt)
        self.assertEqual(lint_errors, [])

    def test_generate_subagent_prompt_custom_role_and_instructions(self):
        """Verify that prompt generation includes custom instructions and role."""
        custom_instructions = "Verify that all AST nodes conform to SysML v2 syntax."
        prompt = generate_subagent_prompt(
            skill=self.skill_auditor,
            target="src/compiler.py",
            role="Adversarial Code Auditor",
            subagent_type="adversarial_auditor",
            instructions=custom_instructions,
            base_dir=PROJECT_ROOT,
        )

        self.assertIn("Adversarial Code Auditor", prompt)
        self.assertIn("adversarial_auditor", prompt)
        self.assertIn(custom_instructions, prompt)

        lint_errors = lint_prompt_text(prompt)
        self.assertEqual(lint_errors, [])

    def test_generate_subagent_prompt_empty_target_rejection(self):
        """Verify that empty or whitespace target strings are rejected."""
        with self.assertRaises(ValueError):
            generate_subagent_prompt(
                skill=self.skill_feature,
                target="",
                base_dir=PROJECT_ROOT,
            )

    def test_generate_subagent_prompt_multi_item_scope_rejection(self):
        """Verify that targets specifying multiple features are rejected by the prompt linter."""
        with self.assertRaises(ValueError) as ctx:
            generate_subagent_prompt(
                skill=self.skill_feature,
                target="FEAT-01 and FEAT-02",
                base_dir=PROJECT_ROOT,
            )
        self.assertIn("multiple Features detected", str(ctx.exception))

    def test_dispatch_subagent_writes_file_explicit_output(self):
        """Verify that dispatch_subagent writes the verified payload to a specified output file."""
        with tempfile.NamedTemporaryFile("w", suffix=".md", delete=False) as tf:
            output_path = tf.name

        try:
            res_path = dispatch_subagent(
                skill=self.skill_feature,
                target=self.target_file,
                output=output_path,
                base_dir=PROJECT_ROOT,
            )
            self.assertEqual(res_path, output_path)
            self.assertTrue(os.path.exists(output_path))
            self.assertGreater(os.path.getsize(output_path), 0)

            with open(output_path, "r", encoding="utf-8") as f:
                content = f.read()

            self.assertIn("view_file", content)
            self.assertIn("PROCEED", content)
            self.assertEqual(lint_prompt_text(content), [])
        finally:
            if os.path.exists(output_path):
                os.remove(output_path)

    def test_dispatch_subagent_default_output_path(self):
        """Verify that dispatch_subagent creates a temporary output file if not specified."""
        res_path = dispatch_subagent(
            skill=self.skill_feature,
            target=self.target_file,
            base_dir=PROJECT_ROOT,
        )
        try:
            self.assertTrue(os.path.exists(res_path))
            self.assertTrue(res_path.startswith("/tmp/subagent_prompt_"))
            self.assertTrue(res_path.endswith(".md"))
            self.assertGreater(os.path.getsize(res_path), 0)
        finally:
            if os.path.exists(res_path):
                os.remove(res_path)

    def test_cli_execution_success(self):
        """Verify CLI execution produces exit code 0 and outputs payload."""
        cmd = [
            sys.executable,
            os.path.join(PROJECT_ROOT, "scripts", "dispatch_subagent.py"),
            "--skill",
            self.skill_feature,
            "--target",
            self.target_file,
            "--role",
            "CLI Subagent Worker",
        ]
        res = subprocess.run(cmd, capture_output=True, text=True)
        self.assertEqual(res.returncode, 0, f"CLI failed: {res.stderr}")
        self.assertIn("Generated subagent prompt payload written to:", res.stdout)
        self.assertIn("CLI Subagent Worker", res.stdout)
        self.assertIn("PROCEED", res.stdout)

    def test_cli_execution_nonexistent_skill_fail(self):
        """Verify CLI execution exits with non-zero when given an invalid skill path."""
        cmd = [
            sys.executable,
            os.path.join(PROJECT_ROOT, "scripts", "dispatch_subagent.py"),
            "--skill",
            "skills/nonexistent/SKILL.md",
            "--target",
            self.target_file,
        ]
        res = subprocess.run(cmd, capture_output=True, text=True)
        self.assertNotEqual(res.returncode, 0)
        self.assertIn("Error dispatching subagent", res.stderr)

    def test_cli_execution_multi_item_target_fail(self):
        """Verify CLI execution exits with non-zero when given a multi-feature target."""
        cmd = [
            sys.executable,
            os.path.join(PROJECT_ROOT, "scripts", "dispatch_subagent.py"),
            "--skill",
            self.skill_feature,
            "--target",
            "FEAT-10 and FEAT-11",
        ]
        res = subprocess.run(cmd, capture_output=True, text=True)
        self.assertNotEqual(res.returncode, 0)
        self.assertIn("Error dispatching subagent", res.stderr)
        self.assertIn("multiple Features detected", res.stderr)


    def test_resolve_classification_explicit_override(self):
        """Verify explicit classification parameter takes top priority."""
        res = resolve_repository_classification(explicit_classification="CUSTOM_CLASSIFICATION")
        self.assertEqual(res, "CUSTOM_CLASSIFICATION")

    def test_resolve_classification_env_var_override(self):
        """Verify DEAP_REPOSITORY_TYPE and REPO_CLASSIFICATION env vars override workspace defaults (Issue #90)."""
        # Test DEAP_REPOSITORY_TYPE
        os.environ["DEAP_REPOSITORY_TYPE"] = "PARENT_DOMAIN_DISTRIBUTION_TEMPLATE"
        try:
            res = resolve_repository_classification()
            self.assertEqual(res, "PARENT_DOMAIN_DISTRIBUTION_TEMPLATE")
        finally:
            os.environ.pop("DEAP_REPOSITORY_TYPE", None)

        # Test REPO_CLASSIFICATION
        os.environ["REPO_CLASSIFICATION"] = "CHILD_DOMAIN_DISTRIBUTION_TEMPLATE"
        try:
            res = resolve_repository_classification()
            self.assertEqual(res, "CHILD_DOMAIN_DISTRIBUTION_TEMPLATE")
        finally:
            os.environ.pop("REPO_CLASSIFICATION", None)

    def test_resolve_classification_from_lineage_json_parent_domain(self):
        """Verify dynamic detection of PARENT_DOMAIN_DISTRIBUTION_TEMPLATE from lineage.json (Issue #87)."""
        temp_ws = tempfile.mkdtemp(prefix="deap_test_ws_")
        try:
            pipe_dir = os.path.join(temp_ws, ".pipeline")
            os.makedirs(pipe_dir, exist_ok=True)
            lineage_data = {
                "self": "DEAP-avionic-flight-safety",
                "tier": 1,
                "role": "PARENT_DOMAIN_DISTRIBUTION_TEMPLATE",
                "upstream": "gintatkinson/DEAP01-spec-core",
            }
            with open(os.path.join(pipe_dir, "lineage.json"), "w", encoding="utf-8") as f:
                json.dump(lineage_data, f)

            res = resolve_repository_classification(base_dir=temp_ws)
            self.assertEqual(res, "PARENT_DOMAIN_DISTRIBUTION_TEMPLATE")
        finally:
            shutil.rmtree(temp_ws, ignore_errors=True)

    def test_resolve_classification_from_lineage_json_child_domain(self):
        """Verify dynamic detection of CHILD_DOMAIN_DISTRIBUTION_TEMPLATE from lineage.json (Issue #87)."""
        temp_ws = tempfile.mkdtemp(prefix="deap_test_ws_")
        try:
            pipe_dir = os.path.join(temp_ws, ".pipeline")
            os.makedirs(pipe_dir, exist_ok=True)
            lineage_data = {
                "self": "DEAP-uas-infrastructure-safety",
                "tier": 2,
                "role": "CHILD_DOMAIN_DISTRIBUTION_TEMPLATE",
                "parent": "gintatkinson/DEAP-avionic-flight-safety",
            }
            with open(os.path.join(pipe_dir, "lineage.json"), "w", encoding="utf-8") as f:
                json.dump(lineage_data, f)

            res = resolve_repository_classification(base_dir=temp_ws)
            self.assertEqual(res, "CHILD_DOMAIN_DISTRIBUTION_TEMPLATE")
        finally:
            shutil.rmtree(temp_ws, ignore_errors=True)

    def test_resolve_classification_from_lineage_json_downstream_workspace(self):
        """Verify dynamic detection of DOWNSTREAM_APPLICATION_WORKSPACE from lineage.json (Issue #87)."""
        temp_ws = tempfile.mkdtemp(prefix="deap_test_ws_")
        try:
            pipe_dir = os.path.join(temp_ws, ".pipeline")
            os.makedirs(pipe_dir, exist_ok=True)
            lineage_data = {
                "self": "UAS-001",
                "tier": 3,
                "role": "DOWNSTREAM_APPLICATION_WORKSPACE",
            }
            with open(os.path.join(pipe_dir, "lineage.json"), "w", encoding="utf-8") as f:
                json.dump(lineage_data, f)

            res = resolve_repository_classification(base_dir=temp_ws)
            self.assertEqual(res, "DOWNSTREAM_APPLICATION_WORKSPACE")
        finally:
            shutil.rmtree(temp_ws, ignore_errors=True)

    def test_resolve_classification_from_constitution_md(self):
        """Verify dynamic detection of classification from .pipeline/constitution.md (Issue #87)."""
        temp_ws = tempfile.mkdtemp(prefix="deap_test_ws_")
        try:
            pipe_dir = os.path.join(temp_ws, ".pipeline")
            os.makedirs(pipe_dir, exist_ok=True)
            constitution_text = """# Constitution
> **Repository Role:** `DOWNSTREAM_APPLICATION_WORKSPACE`
"""
            with open(os.path.join(pipe_dir, "constitution.md"), "w", encoding="utf-8") as f:
                f.write(constitution_text)

            res = resolve_repository_classification(base_dir=temp_ws)
            self.assertEqual(res, "DOWNSTREAM_APPLICATION_WORKSPACE")
        finally:
            shutil.rmtree(temp_ws, ignore_errors=True)

    def test_resolve_classification_downstream_without_upstream_marker(self):
        """Verify workspace without .pipeline/upstream defaults to DOWNSTREAM_APPLICATION_WORKSPACE."""
        temp_ws = tempfile.mkdtemp(prefix="deap_test_ws_")
        try:
            pipe_dir = os.path.join(temp_ws, ".pipeline")
            os.makedirs(pipe_dir, exist_ok=True)
            res = resolve_repository_classification(base_dir=temp_ws)
            self.assertEqual(res, "DOWNSTREAM_APPLICATION_WORKSPACE")
        finally:
            shutil.rmtree(temp_ws, ignore_errors=True)

    def test_generate_subagent_prompt_dynamic_classification_parent_domain(self):
        """Verify prompt generated in parent domain workspace contains PARENT_DOMAIN_DISTRIBUTION_TEMPLATE."""
        temp_ws = tempfile.mkdtemp(prefix="deap_test_ws_")
        try:
            # Setup mock workspace with lineage.json and skill
            pipe_dir = os.path.join(temp_ws, ".pipeline")
            os.makedirs(pipe_dir, exist_ok=True)
            with open(os.path.join(pipe_dir, "lineage.json"), "w", encoding="utf-8") as f:
                json.dump({"role": "PARENT_DOMAIN_DISTRIBUTION_TEMPLATE", "tier": 1}, f)

            prompt = generate_subagent_prompt(
                skill=self.skill_feature,
                target=self.target_file,
                base_dir=temp_ws,
            )

            self.assertIn("Repository Classification: PARENT_DOMAIN_DISTRIBUTION_TEMPLATE", prompt)
            self.assertIn("You are operating within classification `PARENT_DOMAIN_DISTRIBUTION_TEMPLATE`", prompt)
            self.assertEqual(lint_prompt_text(prompt), [])
            preflight_ok, reason = validate_subagent_preflight(prompt)
            self.assertTrue(preflight_ok, f"Preflight rejected prompt: {reason}")
        finally:
            shutil.rmtree(temp_ws, ignore_errors=True)

    def test_generate_subagent_prompt_dynamic_classification_env_override(self):
        """Verify prompt generated with DEAP_REPOSITORY_TYPE override reflects in prompt preamble (Issue #90)."""
        os.environ["DEAP_REPOSITORY_TYPE"] = "CHILD_DOMAIN_DISTRIBUTION_TEMPLATE"
        try:
            prompt = generate_subagent_prompt(
                skill=self.skill_feature,
                target=self.target_file,
                base_dir=PROJECT_ROOT,
            )

            self.assertIn("Repository Classification: CHILD_DOMAIN_DISTRIBUTION_TEMPLATE", prompt)
            self.assertIn("You are operating within classification `CHILD_DOMAIN_DISTRIBUTION_TEMPLATE`", prompt)
            self.assertEqual(lint_prompt_text(prompt), [])
            preflight_ok, reason = validate_subagent_preflight(prompt)
            self.assertTrue(preflight_ok, f"Preflight rejected prompt: {reason}")
        finally:
            os.environ.pop("DEAP_REPOSITORY_TYPE", None)


if __name__ == "__main__":
    unittest.main()

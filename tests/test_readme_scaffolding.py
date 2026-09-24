#!/usr/bin/env python3
"""
Unit and Regression Test Suite for Three-Tier README Scaffolding and Shell Code Fence Hygiene.

Covers verification of:
1. TestUpstreamCompilerReadme:
   - Verifies that DEAP01-spec-core/README.md declares UPSTREAM_SPEC_CORE_COMPILER.
   - Verifies 0 matches for 'python3 -c' or inline monkeypatching scripts.
   - Verifies 0 matches for hardcoded domain clone commands (e.g. DEAP-uas-infrastructure-safety)
     in Section 5 installation instructions.
   - Verifies presence of compiler-focus commands (compile_sysml.py, pytest,
     verify_downstream_baseline.py --no-domain, and propagation commands).
   - Verifies that all internal markdown links and anchor slugs in README.md resolve cleanly.
2. TestDomainDistributionTemplateScaffolding:
   - Runs scripts/install_pipeline.sh on a temporary directory with --role domain-template
     and with auto-detection on DEAP-* directory names.
   - Asserts role declaration: '> **Repository Role:** `DOMAIN_DISTRIBUTION_TEMPLATE`'.
   - Asserts Clean Landing Zone Invariant in Section 1.1.
   - Asserts single-line customer clone command with zero sibling path dependencies.
3. TestCustomerWorkspaceScaffolding:
   - Runs scripts/install_pipeline.sh on a temporary directory with --role customer-project
     and with auto-detection on uav-* directory names.
   - Asserts role declaration: '> **Repository Role:** `DOWNSTREAM_CUSTOMER_PROJECT`'.
   - Asserts ZERO circular self-clone commands ('git clone.*\\.tmp-pipeline' must NOT be present).
   - Asserts project-specific commands: baseline verification (verify_downstream_baseline.py),
     Level 0 ingestion (sysmlv2_ingest.py), and in-place update (bash scripts/install_pipeline.sh .).
   - Asserts idempotence: running install_pipeline.sh . a second time preserves the README.
   - Asserts legacy circular customer README upgrade: legacy circular README is detected and upgraded.
4. TestShellCodeFenceHygiene:
   - Extracts all bash/sh code fences from README.md and scaffolded templates.
   - Validates each with 'bash -n' via subprocess.
   - Verifies zero unescaped parentheses in '#' comments inside bash fences.
   - Verifies zero unquoted angle brackets ('<...>') in executable bash commands.
5. TestActiveGovernanceRuleBundlingAndPromptCatalog:
   - Verifies that install_pipeline.sh compiles .pipeline/ACTIVE_RULES_BUNDLE.md across all tiers.
   - Verifies 100% rule inclusion and valid TOC.
   - Verifies operator prompt catalogs and README initialization mandate reading ACTIVE_RULES_BUNDLE.md.
   - Verifies zero prompt templates reference isolated rule subsets or bare rules/ directory reads.

Primary Commercial Toolchain Integration Context:
  MATLAB / Simulink / Stateflow / Embedded Coder (Model-Based Design, Control Law Synthesis,
  DO-178C C/SPARK Ada code generation).

Pure Schema-Driven Compiler Invariant:
  Zero hardcoded domain concepts. All test runs operate in isolated temporary directories
  outside the workspace repository and leave zero residual state.
"""

import os
import re
import subprocess
import sys
import tempfile
import unittest

PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
INSTALL_SCRIPT = os.path.join(PROJECT_ROOT, "scripts", "install_pipeline.sh")
UPSTREAM_README = os.path.join(PROJECT_ROOT, "README.md")


def github_slugify(heading: str) -> str:
    """Generate GitHub-compatible markdown anchor slug from heading text."""
    # Strip markdown formatting: bold, italic, backticks, links
    cleaned = re.sub(r'[*`_]', '', heading)
    cleaned = re.sub(r'\[([^\]]+)\]\([^)]+\)', r'\1', cleaned)
    s = cleaned.strip().lower()
    s = re.sub(r'[^\w\s-]', '', s)
    s = re.sub(r'[\s]+', '-', s)
    return s


def extract_bash_code_fences(markdown_text: str) -> list[str]:
    """Extract all bash and sh code fence bodies from markdown text."""
    return re.findall(r'```(?:bash|sh)\s*\n(.*?)```', markdown_text, re.DOTALL)


def find_unescaped_parentheses_in_comments(fence: str) -> list[tuple[int, str]]:
    """Return list of (line_number, line) where a '#' comment contains unescaped parens."""
    violations = []
    for line_no, line in enumerate(fence.splitlines(), 1):
        stripped = line.strip()
        if stripped.startswith('#'):
            # Detect any '(' or ')' not preceded by a backslash
            if re.search(r'(?<!\\)[()]', stripped):
                violations.append((line_no, line))
    return violations


def find_unquoted_angle_brackets(line: str) -> list[str]:
    """
    Find occurrences of '<placeholder>' tokens in an executable command line
    that are NOT enclosed inside single or double quotes.
    Ignores heredoc delimiters (e.g. '<< EOF') and process substitutions ('<(').
    """
    in_single = False
    in_double = False
    escaped = False
    code_part = ""

    for char in line:
        if escaped:
            escaped = False
            code_part += char
            continue
        if char == '\\' and not in_single:
            escaped = True
            code_part += char
            continue
        if char == "'" and not in_double:
            in_single = not in_single
            code_part += char
            continue
        if char == '"' and not in_single:
            in_double = not in_double
            code_part += char
            continue
        if char == '#' and not in_single and not in_double:
            # Trailing comment begins
            break
        code_part += char

    # Strip quoted substrings to isolate unquoted code
    unquoted = re.sub(r"'[^']*'", "", code_part)
    unquoted = re.sub(r'"(?:\\.|[^"\\])*"', "", unquoted)

    # Detect angle-bracket placeholder tokens like <path-to-domain-template>
    matches = re.findall(r'(?<!<)<([a-zA-Z0-9_\-\.\/ ]+)>(?!>)', unquoted)
    return matches


class TestUpstreamCompilerReadme(unittest.TestCase):
    """Verifies that DEAP01-spec-core/README.md maintains upstream compiler focus and cleanliness."""

    def setUp(self):
        self.assertTrue(os.path.isfile(UPSTREAM_README), f"README.md not found at {UPSTREAM_README}")
        with open(UPSTREAM_README, "r", encoding="utf-8") as f:
            self.content = f.read()

    def test_upstream_readme_repository_classification(self):
        """Verifies that README.md declares UPSTREAM_SPEC_CORE_COMPILER repository role."""
        self.assertIn(
            "> **Repository Role:** `UPSTREAM_SPEC_CORE_COMPILER`",
            self.content,
            "README.md must declare UPSTREAM_SPEC_CORE_COMPILER role."
        )

    def test_upstream_readme_no_inline_python_scripts(self):
        """Verifies 0 matches for 'python3 -c' or inline monkeypatching scripts in README.md."""
        self.assertNotIn("python3 -c", self.content, "Inline 'python3 -c' script found in README.md")
        self.assertNotIn("python -c", self.content, "Inline 'python -c' script found in README.md")

        # Verify no inline python code fences modifying repo files
        py_fences = re.findall(r'```python\s*\n(.*?)```', self.content, re.DOTALL)
        for fence in py_fences:
            self.assertNotIn(
                "open('AGENTS.md'",
                fence,
                "Found inline monkeypatching script modifying AGENTS.md in README.md"
            )

    def test_upstream_readme_no_domain_clone_commands(self):
        """Verifies 0 matches for hardcoded domain clone commands in Section 5 installation instructions."""
        # Extract Section 5
        sec5_match = re.search(r'## 5\..*?(?=## 6\.|\Z)', self.content, re.DOTALL)
        self.assertIsNotNone(sec5_match, "Section 5 not found in README.md")
        sec5_text = sec5_match.group(0)

        # Assert no executable git clone command targets DEAP-uas-infrastructure-safety or other domains
        sec5_fences = extract_bash_code_fences(sec5_text)
        for fence in sec5_fences:
            for line in fence.splitlines():
                if "git clone" in line:
                    self.assertNotIn(
                        "DEAP-uas-infrastructure-safety",
                        line,
                        f"Found forbidden domain clone command in Section 5 code fence: {line}"
                    )
                    self.assertIn(
                        "DEAP01-spec-core",
                        line,
                        f"Section 5 clone command must target upstream compiler, got: {line}"
                    )

        # Assert no un-fenced hardcoded customer onboarding clone commands in Section 5
        self.assertIsNone(
            re.search(r'git clone.*DEAP-uas-infrastructure-safety\.git.*\.tmp-pipeline', sec5_text),
            "Found hardcoded domain onboarding clone command in Section 5."
        )

    def test_upstream_readme_compiler_focus_commands(self):
        """Verifies presence of compile_sysml.py, pytest, verify_downstream_baseline.py --no-domain, and propagation commands."""
        # SysML compilation command
        self.assertIn("python3 scripts/compile_sysml.py --compile", self.content)

        # Test runner execution
        self.assertTrue(
            "pytest" in self.content or "python3 -m pytest" in self.content,
            "pytest command must be documented in README.md"
        )

        # Baseline verification command
        self.assertIn("python3 scripts/verify_downstream_baseline.py --no-domain", self.content)

        # Local propagation command
        self.assertIn('bash scripts/install_pipeline.sh "<path-to-domain-template>"', self.content)

        # Remote bootstrap propagation command
        self.assertIn("/tmp/deap_compiler", self.content)
        self.assertIn(
            "git clone https://github.com/gintatkinson/DEAP01-spec-core.git /tmp/deap_compiler && "
            "bash /tmp/deap_compiler/scripts/install_pipeline.sh . && rm -rf /tmp/deap_compiler",
            self.content
        )

    def test_upstream_readme_anchor_links(self):
        """Verifies all internal markdown links and anchor slugs in README.md resolve cleanly."""
        # Collect heading slugs
        slugs = set()
        for line in self.content.splitlines():
            m = re.match(r'^(#{1,6})\s+(.+)$', line.strip())
            if m:
                slugs.add(github_slugify(m.group(2)))

        # Find all markdown links [text](target)
        links = re.findall(r'\[([^\]]+)\]\(([^)]+)\)', self.content)
        self.assertGreater(len(links), 0, "No markdown links found in README.md")

        for text, target in links:
            if target.startswith("#"):
                anchor = target[1:]
                self.assertIn(
                    anchor,
                    slugs,
                    f"Anchor link '#{anchor}' (from [{text}]({target})) does not match any heading slug in README.md"
                )
            elif not target.startswith(("http://", "https://", "mailto:")):
                parts = target.split("#")
                rel_path = parts[0]
                if rel_path:
                    full_path = os.path.join(PROJECT_ROOT, rel_path)
                    self.assertTrue(
                        os.path.exists(full_path),
                        f"Relative path '{rel_path}' referenced in [{text}]({target}) does not exist on disk"
                    )


class TestDomainDistributionTemplateScaffolding(unittest.TestCase):
    """Verifies that install_pipeline.sh properly scaffolds Tier 1 Domain Distribution Templates."""

    def setUp(self):
        self.assertTrue(os.path.isfile(INSTALL_SCRIPT), f"Installer not found at {INSTALL_SCRIPT}")
        self.assertTrue(os.access(INSTALL_SCRIPT, os.X_OK), f"Installer is not executable: {INSTALL_SCRIPT}")

    def _run_installer(self, *args, cwd=None):
        cmd = ["bash", INSTALL_SCRIPT] + list(args)
        return subprocess.run(
            cmd,
            cwd=cwd or PROJECT_ROOT,
            capture_output=True,
            text=True
        )

    def test_domain_template_scaffolding_explicit_role(self):
        """Runs install_pipeline.sh on a temporary directory with --role domain-template, asserts README scaffolding."""
        with tempfile.TemporaryDirectory() as temp_dir:
            proc = self._run_installer(temp_dir, "--role", "domain-template")
            self.assertEqual(proc.returncode, 0, f"Installer failed. stderr: {proc.stderr}")

            readme_path = os.path.join(temp_dir, "README.md")
            self.assertTrue(os.path.isfile(readme_path), "README.md was not generated")
            with open(readme_path, "r", encoding="utf-8") as f:
                content = f.read()

            self.assertIn("> **Repository Role:** `DOMAIN_DISTRIBUTION_TEMPLATE`", content)
            self.assertIn("### 1.1 Clean Landing Zone Invariant", content)

            # Assert single-line customer clone command
            expected_clone_pattern = (
                r'git clone\s+https://github\.com/[^\s]+\.git\s+\./\.tmp-pipeline\s+&&\s+'
                r'bash\s+\./\.tmp-pipeline/scripts/install_pipeline\.sh\s+\.\s+&&\s+rm\s+-rf\s+\./\.tmp-pipeline'
            )
            self.assertIsNotNone(
                re.search(expected_clone_pattern, content),
                f"Customer onboarding command not found in scaffolded README:\n{content}"
            )

    def test_domain_template_scaffolding_auto_detection(self):
        """Runs install_pipeline.sh on a directory named DEAP-* and asserts auto-detection of DOMAIN_DISTRIBUTION_TEMPLATE."""
        with tempfile.TemporaryDirectory() as temp_dir:
            domain_dir = os.path.join(temp_dir, "DEAP-uas-infrastructure-safety")
            os.makedirs(domain_dir)

            proc = self._run_installer(domain_dir)
            self.assertEqual(proc.returncode, 0, f"Installer failed. stderr: {proc.stderr}")

            readme_path = os.path.join(domain_dir, "README.md")
            self.assertTrue(os.path.isfile(readme_path), "README.md was not generated")
            with open(readme_path, "r", encoding="utf-8") as f:
                content = f.read()

            self.assertIn("> **Repository Role:** `DOMAIN_DISTRIBUTION_TEMPLATE`", content)
            self.assertIn("### 1.1 Clean Landing Zone Invariant", content)

            # Assert domain-specific customer clone command
            expected_clone_cmd = (
                "git clone https://github.com/gintatkinson/DEAP-uas-infrastructure-safety.git ./.tmp-pipeline && "
                "bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline"
            )
            self.assertIn(expected_clone_cmd, content)

    def test_domain_template_scaffolding_clean_landing_zones(self):
        """Asserts that the scaffolded domain template README documents the clean landing zone invariant in Section 1.1."""
        with tempfile.TemporaryDirectory() as temp_dir:
            proc = self._run_installer(temp_dir, "--role", "domain-template")
            self.assertEqual(proc.returncode, 0, f"Installer failed. stderr: {proc.stderr}")

            readme_path = os.path.join(temp_dir, "README.md")
            with open(readme_path, "r", encoding="utf-8") as f:
                content = f.read()

            self.assertIn("### 1.1 Clean Landing Zone Invariant", content)
            self.assertIn("schema/", content)
            self.assertIn("docs/epics/", content)
            self.assertIn("docs/features/", content)
            self.assertIn("docs/user-stories/", content)
            self.assertIn("docs/use-cases/", content)
            self.assertIn(".gitkeep", content)

    def test_domain_template_scaffolding_customer_clone_command_zero_sibling_dependencies(self):
        """Asserts that the customer clone command operates strictly with zero sibling path dependencies (../...)."""
        with tempfile.TemporaryDirectory() as temp_dir:
            proc = self._run_installer(temp_dir, "--role", "domain-template")
            self.assertEqual(proc.returncode, 0, f"Installer failed. stderr: {proc.stderr}")

            readme_path = os.path.join(temp_dir, "README.md")
            with open(readme_path, "r", encoding="utf-8") as f:
                content = f.read()

            fences = extract_bash_code_fences(content)
            clone_lines = [line for fence in fences for line in fence.splitlines() if "git clone" in line]
            self.assertGreater(len(clone_lines), 0, "No git clone command in domain template fences")

            for line in clone_lines:
                self.assertNotIn("../", line, f"Found forbidden sibling path dependency in clone command: {line}")
                self.assertIn(".tmp-pipeline", line)


class TestCustomerWorkspaceScaffolding(unittest.TestCase):
    """Verifies that install_pipeline.sh properly scaffolds Tier 2 Customer Application Workspaces."""

    def setUp(self):
        self.assertTrue(os.path.isfile(INSTALL_SCRIPT), f"Installer not found at {INSTALL_SCRIPT}")
        self.assertTrue(os.access(INSTALL_SCRIPT, os.X_OK), f"Installer is not executable: {INSTALL_SCRIPT}")

    def _run_installer(self, *args, cwd=None):
        cmd = ["bash", INSTALL_SCRIPT] + list(args)
        return subprocess.run(
            cmd,
            cwd=cwd or PROJECT_ROOT,
            capture_output=True,
            text=True
        )

    def test_customer_workspace_scaffolding_explicit_role(self):
        """Runs install_pipeline.sh on a temporary directory with --role customer-project, asserts README scaffolding."""
        with tempfile.TemporaryDirectory() as temp_dir:
            proc = self._run_installer(temp_dir, "--role", "customer-project")
            self.assertEqual(proc.returncode, 0, f"Installer failed. stderr: {proc.stderr}")

            readme_path = os.path.join(temp_dir, "README.md")
            self.assertTrue(os.path.isfile(readme_path), "README.md was not generated")
            with open(readme_path, "r", encoding="utf-8") as f:
                content = f.read()

            self.assertIn("> **Repository Role:** `DOWNSTREAM_CUSTOMER_PROJECT`", content)

            # Assert ZERO circular clone commands
            self.assertIsNone(
                re.search(r'git clone.*\.tmp-pipeline', content),
                "Found forbidden circular clone command in customer workspace README"
            )

            # Assert project-specific commands
            self.assertIn("python3 scripts/verify_downstream_baseline.py --no-domain", content)
            self.assertIn("python3 skills/spec-orchestrator/scripts/sysmlv2_ingest.py", content)
            self.assertIn("bash scripts/install_pipeline.sh .", content)

    def test_customer_workspace_scaffolding_auto_detection(self):
        """Runs install_pipeline.sh on a directory named uav-* and asserts auto-detection of DOWNSTREAM_CUSTOMER_PROJECT."""
        with tempfile.TemporaryDirectory() as temp_dir:
            cust_dir = os.path.join(temp_dir, "uav-011")
            os.makedirs(cust_dir)

            proc = self._run_installer(cust_dir)
            self.assertEqual(proc.returncode, 0, f"Installer failed. stderr: {proc.stderr}")

            readme_path = os.path.join(cust_dir, "README.md")
            self.assertTrue(os.path.isfile(readme_path), "README.md was not generated")
            with open(readme_path, "r", encoding="utf-8") as f:
                content = f.read()

            self.assertIn("> **Repository Role:** `DOWNSTREAM_CUSTOMER_PROJECT`", content)
            self.assertIsNone(
                re.search(r'git clone.*\.tmp-pipeline', content),
                "Found forbidden circular clone command in auto-detected customer workspace README"
            )
            self.assertIn("python3 scripts/verify_downstream_baseline.py --no-domain", content)
            self.assertIn("python3 skills/spec-orchestrator/scripts/sysmlv2_ingest.py", content)
            self.assertIn("bash scripts/install_pipeline.sh .", content)

    def test_customer_workspace_scaffolding_auto_detection_with_domain_url(self):
        """Runs install_pipeline.sh on uav-* with --domain-url and asserts DOWNSTREAM_CUSTOMER_PROJECT with no circular clone."""
        with tempfile.TemporaryDirectory() as temp_dir:
            cust_dir = os.path.join(temp_dir, "uav-011")
            os.makedirs(cust_dir)

            proc = self._run_installer(
                cust_dir,
                "--domain-url", "https://github.com/gintatkinson/DEAP-uas-infrastructure-safety.git"
            )
            self.assertEqual(proc.returncode, 0, f"Installer failed. stderr: {proc.stderr}")
            self.assertIn("Target repository role: DOWNSTREAM_CUSTOMER_PROJECT", proc.stdout)

            readme_path = os.path.join(cust_dir, "README.md")
            self.assertTrue(os.path.isfile(readme_path), "README.md was not generated")
            with open(readme_path, "r", encoding="utf-8") as f:
                content = f.read()

            self.assertIn("> **Repository Role:** `DOWNSTREAM_CUSTOMER_PROJECT`", content)
            self.assertIsNone(
                re.search(r'git clone.*\.tmp-pipeline', content),
                "Found forbidden circular clone command in customer workspace README"
            )
            self.assertIn("python3 scripts/verify_downstream_baseline.py --no-domain", content)
            self.assertIn("python3 skills/spec-orchestrator/scripts/sysmlv2_ingest.py", content)
            self.assertIn("bash scripts/install_pipeline.sh .", content)

    def test_customer_workspace_scaffolding_no_circular_clone(self):
        """Verifies that no circular clone command exists in customer workspace README."""
        with tempfile.TemporaryDirectory() as temp_dir:
            proc = self._run_installer(temp_dir, "--role", "customer-project")
            self.assertEqual(proc.returncode, 0, f"Installer failed. stderr: {proc.stderr}")

            readme_path = os.path.join(temp_dir, "README.md")
            with open(readme_path, "r", encoding="utf-8") as f:
                content = f.read()

            # Ensure no command tells the user inside the project to clone itself
            self.assertNotIn("git clone", content)

    def test_customer_workspace_scaffolding_idempotence(self):
        """Asserts idempotence: running install_pipeline.sh . a second time preserves customer workspace README."""
        with tempfile.TemporaryDirectory() as temp_dir:
            cust_dir = os.path.join(temp_dir, "uav-011")
            os.makedirs(cust_dir)

            proc1 = self._run_installer(cust_dir, "--role", "customer-project")
            self.assertEqual(proc1.returncode, 0, f"Initial install failed. stderr: {proc1.stderr}")

            readme_path = os.path.join(cust_dir, "README.md")
            with open(readme_path, "r", encoding="utf-8") as f:
                initial_content = f.read()

            # Second execution in-place
            proc2 = self._run_installer(".", cwd=cust_dir)
            self.assertEqual(proc2.returncode, 0, f"In-place install failed. stderr: {proc2.stderr}")

            with open(readme_path, "r", encoding="utf-8") as f:
                second_content = f.read()

            self.assertEqual(
                initial_content,
                second_content,
                "In-place second execution must be idempotent and preserve README.md"
            )

    def test_customer_workspace_legacy_circular_upgrade(self):
        """Asserts legacy circular customer README is detected and upgraded to DOWNSTREAM_CUSTOMER_PROJECT."""
        with tempfile.TemporaryDirectory() as temp_dir:
            cust_dir = os.path.join(temp_dir, "uav-011")
            os.makedirs(cust_dir)

            # Initial install
            proc1 = self._run_installer(cust_dir, "--role", "customer-project")
            self.assertEqual(proc1.returncode, 0, f"Initial install failed. stderr: {proc1.stderr}")

            # Plant legacy circular README
            readme_path = os.path.join(cust_dir, "README.md")
            legacy_content = (
                "# uav-011 -- Legacy Downstream Project\n\n"
                "> **Repository Role:** `DOWNSTREAM_APPLICATION_WORKSPACE`  \n\n"
                "## 3. Customer Project Onboarding & Agent Initialization Sequence\n\n"
                "```bash\n"
                "git clone https://gitlab.com/uav/uav-011.git ./.tmp-pipeline && "
                "bash ./.tmp-pipeline/scripts/install_pipeline.sh . && rm -rf ./.tmp-pipeline\n"
                "```\n"
            )
            with open(readme_path, "w", encoding="utf-8") as f:
                f.write(legacy_content)

            # Run in-place update
            proc2 = self._run_installer(".", cwd=cust_dir)
            self.assertEqual(proc2.returncode, 0, f"In-place update failed. stderr: {proc2.stderr}")

            with open(readme_path, "r", encoding="utf-8") as f:
                upgraded_content = f.read()

            self.assertIn(
                "> **Repository Role:** `DOWNSTREAM_CUSTOMER_PROJECT`",
                upgraded_content,
                "Role was not upgraded to DOWNSTREAM_CUSTOMER_PROJECT"
            )
            self.assertIsNone(
                re.search(r'git clone.*\.tmp-pipeline', upgraded_content),
                "Legacy circular clone command was not removed during upgrade"
            )
            self.assertIn("python3 scripts/verify_downstream_baseline.py --no-domain", upgraded_content)
            self.assertIn("python3 skills/spec-orchestrator/scripts/sysmlv2_ingest.py", upgraded_content)
            self.assertIn("bash scripts/install_pipeline.sh .", upgraded_content)


class TestShellCodeFenceHygiene(unittest.TestCase):
    """Verifies bash/sh syntax validity and hygiene across upstream README and scaffolded templates."""

    def _assert_fence_hygiene(self, text: str, source_label: str):
        """Helper to assert bash -n syntax, comment parentheses, and angle brackets for all bash fences."""
        fences = extract_bash_code_fences(text)
        self.assertGreater(len(fences), 0, f"No bash/sh code fences found in {source_label}")

        for i, fence in enumerate(fences, 1):
            # 1. Syntax check with bash -n
            proc = subprocess.run(
                ["bash", "-n"],
                input=fence,
                text=True,
                capture_output=True
            )
            self.assertEqual(
                proc.returncode,
                0,
                f"bash -n failed on code fence #{i} in {source_label}:\n{proc.stderr}\nCode fence:\n{fence}"
            )

            # 2. Check unescaped parentheses in '#' comments
            comment_violations = find_unescaped_parentheses_in_comments(fence)
            self.assertEqual(
                comment_violations,
                [],
                f"Found unescaped parentheses in comment(s) in code fence #{i} in {source_label}: {comment_violations}"
            )

            # 3. Check unquoted angle brackets in executable command lines
            for line_no, line in enumerate(fence.splitlines(), 1):
                stripped = line.strip()
                if stripped and not stripped.startswith('#'):
                    bracket_matches = find_unquoted_angle_brackets(line)
                    self.assertEqual(
                        bracket_matches,
                        [],
                        f"Found unquoted angle bracket placeholder(s) {bracket_matches} on line {line_no} "
                        f"in code fence #{i} in {source_label}:\n{line}"
                    )

    def test_upstream_readme_shell_code_fences_hygiene(self):
        """Verifies all bash/sh code fences in DEAP01-spec-core/README.md are syntactically valid and clean."""
        with open(UPSTREAM_README, "r", encoding="utf-8") as f:
            content = f.read()
        self._assert_fence_hygiene(content, "DEAP01-spec-core/README.md")

    def test_scaffolded_domain_template_shell_hygiene(self):
        """Verifies all bash/sh code fences in scaffolded Domain Distribution Template README.md are syntactically valid and clean."""
        with tempfile.TemporaryDirectory() as temp_dir:
            proc = subprocess.run(
                ["bash", INSTALL_SCRIPT, temp_dir, "--role", "domain-template"],
                capture_output=True,
                text=True
            )
            self.assertEqual(proc.returncode, 0, f"Installer failed. stderr: {proc.stderr}")

            readme_path = os.path.join(temp_dir, "README.md")
            with open(readme_path, "r", encoding="utf-8") as f:
                content = f.read()

            self._assert_fence_hygiene(content, "scaffolded Domain Template README.md")

    def test_scaffolded_customer_workspace_shell_hygiene(self):
        """Verifies all bash/sh code fences in scaffolded Customer Workspace README.md are syntactically valid and clean."""
        with tempfile.TemporaryDirectory() as temp_dir:
            proc = subprocess.run(
                ["bash", INSTALL_SCRIPT, temp_dir, "--role", "customer-project"],
                capture_output=True,
                text=True
            )
            self.assertEqual(proc.returncode, 0, f"Installer failed. stderr: {proc.stderr}")

            readme_path = os.path.join(temp_dir, "README.md")
            with open(readme_path, "r", encoding="utf-8") as f:
                content = f.read()

            self._assert_fence_hygiene(content, "scaffolded Customer Workspace README.md")


class TestActiveGovernanceRuleBundlingAndPromptCatalog(unittest.TestCase):
    """
    Verifies that install_pipeline.sh automatically compiles active governance rules
    into .pipeline/ACTIVE_RULES_BUNDLE.md across all tiers, and enforces that generated
    README operator prompts and prompt catalogs mandate reading the consolidated bundle
    without pointing to isolated rule subsets or bare rules/ directory reads (Issue #368).
    """

    def setUp(self):
        self.assertTrue(os.path.isfile(INSTALL_SCRIPT), f"Installer not found at {INSTALL_SCRIPT}")
        self.assertTrue(os.access(INSTALL_SCRIPT, os.X_OK), f"Installer is not executable: {INSTALL_SCRIPT}")
        self.rules_dir = os.path.join(PROJECT_ROOT, "rules")
        self.rule_files = sorted([f for f in os.listdir(self.rules_dir) if f.endswith(".md")])
        self.assertGreater(len(self.rule_files), 0, "No markdown rule files found in rules/")

    def _run_installer(self, *args, cwd=None):
        cmd = ["bash", INSTALL_SCRIPT] + list(args)
        return subprocess.run(
            cmd,
            cwd=cwd or PROJECT_ROOT,
            capture_output=True,
            text=True
        )

    def _verify_bundle_completeness(self, bundle_path: str):
        """Helper to assert that ACTIVE_RULES_BUNDLE.md exists, has valid TOC, and includes 100% of all rules."""
        self.assertTrue(os.path.isfile(bundle_path), f"Bundle not found at {bundle_path}")
        with open(bundle_path, "r", encoding="utf-8") as f:
            bundle_content = f.read()

        # 1. Header and Notice block
        self.assertIn("# ACTIVE RULES BUNDLE — Consolidated Governance Manifest", bundle_content)
        self.assertIn("scripts/install_pipeline.sh", bundle_content)
        self.assertIn("## Table of Contents", bundle_content)

        # 2. Table of Contents entries
        for rule_file in self.rule_files:
            rule_slug = re.sub(r'^-|-$', '', re.sub(r'-+', '-', re.sub(r'[^a-z0-9]', '-', rule_file.lower())))
            toc_entry = f"[{rule_file}](#rule-{rule_slug})"
            self.assertIn(toc_entry, bundle_content, f"Missing TOC entry '{toc_entry}' in bundle")

        # 3. Rule anchors and 100% unabridged rule content
        for rule_file in self.rule_files:
            rule_slug = re.sub(r'^-|-$', '', re.sub(r'-+', '-', re.sub(r'[^a-z0-9]', '-', rule_file.lower())))
            self.assertIn(f'<a id="rule-{rule_slug}"></a>', bundle_content)
            self.assertIn(f"## Rule: {rule_file}", bundle_content)

            rule_src_path = os.path.join(self.rules_dir, rule_file)
            with open(rule_src_path, "r", encoding="utf-8") as rf:
                rule_text = rf.read()
            self.assertIn(
                rule_text,
                bundle_content,
                f"Rule {rule_file} content is not completely included in ACTIVE_RULES_BUNDLE.md"
            )

        # 4. Delimiters
        self.assertIn("\n---\n", bundle_content)

    def test_domain_template_rule_bundle_generation(self):
        """Verifies that install_pipeline.sh creates .pipeline/ACTIVE_RULES_BUNDLE.md in domain templates."""
        with tempfile.TemporaryDirectory() as temp_dir:
            proc = self._run_installer(temp_dir, "--role", "domain-template")
            self.assertEqual(proc.returncode, 0, f"Installer failed. stderr: {proc.stderr}")
            bundle_path = os.path.join(temp_dir, ".pipeline", "ACTIVE_RULES_BUNDLE.md")
            self._verify_bundle_completeness(bundle_path)

    def test_customer_workspace_rule_bundle_generation(self):
        """Verifies that install_pipeline.sh creates .pipeline/ACTIVE_RULES_BUNDLE.md in customer workspaces."""
        with tempfile.TemporaryDirectory() as temp_dir:
            proc = self._run_installer(temp_dir, "--role", "customer-project")
            self.assertEqual(proc.returncode, 0, f"Installer failed. stderr: {proc.stderr}")
            bundle_path = os.path.join(temp_dir, ".pipeline", "ACTIVE_RULES_BUNDLE.md")
            self._verify_bundle_completeness(bundle_path)

    def test_domain_template_readme_mandates_active_rules_bundle(self):
        """Verifies that Domain Template README Section 3.2 Step 3 explicitly mandates reading ACTIVE_RULES_BUNDLE.md."""
        with tempfile.TemporaryDirectory() as temp_dir:
            proc = self._run_installer(temp_dir, "--role", "domain-template")
            self.assertEqual(proc.returncode, 0, f"Installer failed. stderr: {proc.stderr}")
            readme_path = os.path.join(temp_dir, "README.md")
            with open(readme_path, "r", encoding="utf-8") as f:
                content = f.read()

            expected_step = (
                "3. **Load Governance Rules**: Execute `view_file` on `.pipeline/ACTIVE_RULES_BUNDLE.md` "
                "to ingest the complete, consolidated suite of active governance rules in a single read "
                "(covering dual-track MBD, SysML SSOT completeness, role boundary locks, and TDD mandates)."
            )
            self.assertIn(expected_step, content)
            self.assertNotIn("Ingest `AGENTS.md` and `rules/`", content)

    def test_customer_workspace_readme_mandates_active_rules_bundle(self):
        """Verifies that Customer Workspace README Section 3.3 Step 3 explicitly mandates reading ACTIVE_RULES_BUNDLE.md."""
        with tempfile.TemporaryDirectory() as temp_dir:
            proc = self._run_installer(temp_dir, "--role", "customer-project")
            self.assertEqual(proc.returncode, 0, f"Installer failed. stderr: {proc.stderr}")
            readme_path = os.path.join(temp_dir, "README.md")
            with open(readme_path, "r", encoding="utf-8") as f:
                content = f.read()

            expected_step = (
                "3. **Load Governance Rules**: Execute `view_file` on `.pipeline/ACTIVE_RULES_BUNDLE.md` "
                "to ingest the complete, consolidated suite of active governance rules in a single read "
                "(covering dual-track MBD, SysML SSOT completeness, role boundary locks, and TDD mandates)."
            )
            self.assertIn(expected_step, content)
            self.assertNotIn("Ingest `AGENTS.md` and `rules/`", content)

    def test_readme_prompt_templates_no_isolated_rule_subsets(self):
        """Verifies that no prompt templates in generated READMEs point to isolated rule subsets or bare rules/ directory reads."""
        for role in ["domain-template", "customer-project"]:
            with self.subTest(role=role):
                with tempfile.TemporaryDirectory() as temp_dir:
                    proc = self._run_installer(temp_dir, "--role", role)
                    self.assertEqual(proc.returncode, 0, f"Installer failed for {role}. stderr: {proc.stderr}")
                    readme_path = os.path.join(temp_dir, "README.md")
                    with open(readme_path, "r", encoding="utf-8") as f:
                        content = f.read()

                    # Extract Section 4 (Operator Prompt Catalog)
                    sec4_match = re.search(r'## 4\..*?(?=## 5\.|\Z)', content, re.DOTALL)
                    self.assertIsNotNone(sec4_match, f"Section 4 not found in {role} README")
                    sec4_text = sec4_match.group(0)

                    # Assert no prompt template points to isolated rule files like rules/dual-track-mbd-verification.md
                    self.assertNotIn("rules/dual-track-mbd-verification.md", sec4_text)
                    self.assertNotIn("rules/sysml-ssot-completeness.md", sec4_text)

                    # Assert no prompt template instructs reading bare rules/ directory
                    self.assertNotIn("`rules/`", sec4_text)
                    self.assertNotIn("rules/*.md", sec4_text)

                    # Assert prompt templates reference ACTIVE_RULES_BUNDLE.md in governance preambles
                    self.assertIn(".pipeline/ACTIVE_RULES_BUNDLE.md", sec4_text)

    def test_active_rules_bundle_in_place_idempotence(self):
        """Verifies that running install_pipeline.sh in-place preserves and updates ACTIVE_RULES_BUNDLE.md identically."""
        with tempfile.TemporaryDirectory() as temp_dir:
            cust_dir = os.path.join(temp_dir, "uav-011")
            os.makedirs(cust_dir)

            proc1 = self._run_installer(cust_dir, "--role", "customer-project")
            self.assertEqual(proc1.returncode, 0, f"First install failed. stderr: {proc1.stderr}")

            bundle_path = os.path.join(cust_dir, ".pipeline", "ACTIVE_RULES_BUNDLE.md")
            with open(bundle_path, "r", encoding="utf-8") as f:
                content_first = f.read()

            proc2 = self._run_installer(".", cwd=cust_dir)
            self.assertEqual(proc2.returncode, 0, f"In-place install failed. stderr: {proc2.stderr}")

            with open(bundle_path, "r", encoding="utf-8") as f:
                content_second = f.read()

            self.assertEqual(content_first, content_second, "In-place execution must produce identical rule bundle")


if __name__ == "__main__":
    unittest.main()

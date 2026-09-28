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
   - Verifies that Section 5.5 contains 0 occurrences of deprecated 'Gemini CLI' (refs #379).
   - Verifies that Section 5.6 contains the Universal Agent Governance & Initialization Sequence
     and that steps 0-7 are not misplaced in Section 5.5 (refs #381).
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

from __future__ import annotations

import os
import re
import subprocess
import sys
import tempfile
from typing import Union, Optional, List
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
    s = re.sub(r'\s', '-', s)
    return s


class MarkdownASTNode:
    """Represents a node in the Markdown structural AST."""
    def __init__(
        self,
        node_type: str,
        title: str = "",
        level: int = 0,
        content: str = "",
        info: str = "",
        tag: str = ""
    ):
        self.node_type = node_type  # 'root', 'section', 'fence', 'text'
        self.title = title
        self.level = level
        self.content = content
        self.info = info  # language/info string for code fences (e.g. 'bash', 'mermaid')
        self.tag = tag    # HTML/AST tag name (e.g. 'h1', 'h2')
        self.children: list['MarkdownASTNode'] = []
        self.parent: MarkdownASTNode | None = None

    def add_child(self, child: 'MarkdownASTNode'):
        child.parent = self
        self.children.append(child)

    def find_sections(self, pattern: str | re.Pattern) -> list['MarkdownASTNode']:
        """Recursively find section nodes whose title matches pattern."""
        results = []
        if self.node_type == 'section':
            if isinstance(pattern, re.Pattern):
                if pattern.search(self.title):
                    results.append(self)
            else:
                if pattern.lower() in self.title.lower():
                    results.append(self)
        for child in self.children:
            results.extend(child.find_sections(pattern))
        return results

    def find_fences(self, info_tag: str | None = None) -> list['MarkdownASTNode']:
        """Recursively find code fence nodes, optionally filtered by language tag."""
        results = []
        if self.node_type == 'fence':
            if info_tag is None:
                results.append(self)
            elif info_tag in ('bash', 'sh'):
                if self.info in ('bash', 'sh'):
                    results.append(self)
            elif self.info == info_tag:
                results.append(self)
        for child in self.children:
            results.extend(child.find_fences(info_tag))
        return results

    def get_all_text(self) -> str:
        """Return all text content within this subtree."""
        parts = [self.title, self.content]
        for child in self.children:
            parts.append(child.get_all_text())
        return "\n".join(p for p in parts if p)


def parse_markdown_ast(text: str) -> MarkdownASTNode:
    """
    Parse markdown text into a structural CommonMark AST hierarchy.
    Uses markdown_it when available, with a resilient CommonMark AST fallback.
    """
    try:
        import markdown_it
        md = markdown_it.MarkdownIt()
        tokens = md.parse(text)
        root = MarkdownASTNode('root', level=0)
        stack = [root]
        i = 0
        while i < len(tokens):
            tok = tokens[i]
            if tok.type == 'heading_open':
                level = int(tok.tag[1:])
                title = tokens[i + 1].content if i + 1 < len(tokens) and tokens[i + 1].type == 'inline' else ''
                sec = MarkdownASTNode('section', title=title, level=level, tag=tok.tag)
                while len(stack) > 1 and stack[-1].level >= level:
                    stack.pop()
                stack[-1].add_child(sec)
                stack.append(sec)
                i += 2
            elif tok.type == 'fence':
                fence_node = MarkdownASTNode('fence', content=tok.content, info=tok.info.strip())
                stack[-1].add_child(fence_node)
            elif tok.type == 'inline':
                stack[-1].add_child(MarkdownASTNode('text', content=tok.content))
            i += 1
        return root
    except ImportError:
        root = MarkdownASTNode('root', level=0)
        stack = [root]
        lines = text.splitlines()
        idx = 0
        in_fence = False
        fence_info = ""
        fence_lines = []
        while idx < len(lines):
            line = lines[idx]
            if not in_fence and line.startswith("```"):
                in_fence = True
                fence_info = line[3:].strip()
                fence_lines = []
            elif in_fence and line.startswith("```"):
                in_fence = False
                fence_node = MarkdownASTNode('fence', content="\n".join(fence_lines) + "\n", info=fence_info)
                stack[-1].add_child(fence_node)
            elif in_fence:
                fence_lines.append(line)
            else:
                m = re.match(r'^(#{1,6})\s+(.+)$', line.strip())
                if m:
                    level = len(m.group(1))
                    title = m.group(2).strip()
                    sec = MarkdownASTNode('section', title=title, level=level, tag=f'h{level}')
                    while len(stack) > 1 and stack[-1].level >= level:
                        stack.pop()
                    stack[-1].add_child(sec)
                    stack.append(sec)
                elif line.strip():
                    stack[-1].add_child(MarkdownASTNode('text', content=line))
            idx += 1
        return root


def extract_bash_code_fences(markdown_text: str) -> list[str]:
    """Extract all bash and sh code fence bodies from markdown text using CommonMark AST."""
    ast = parse_markdown_ast(markdown_text)
    return [f.content for f in ast.find_fences("bash")]


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
        # Extract Section 5 via CommonMark AST
        ast = parse_markdown_ast(self.content)
        sec5_nodes = ast.find_sections("5. Installation & Developer Quick-Start Guide")
        self.assertTrue(len(sec5_nodes) > 0, "Section 5 not found in README.md AST")
        sec5_node = sec5_nodes[0]
        sec5_text = sec5_node.get_all_text()

        # Assert no executable git clone command targets DEAP-uas-infrastructure-safety or other domains
        sec5_fences = [f.content for f in sec5_node.find_fences("bash")]
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

    def test_upstream_readme_section_1_heading_order_and_hierarchy(self):
        """Verifies that Section 1.1 precedes Section 1.2 and both are valid H3 subsections of Section 1."""
        ast = parse_markdown_ast(self.content)
        sec1_nodes = ast.find_sections("1. System Overview")
        self.assertTrue(len(sec1_nodes) > 0, "Section 1 not found in README.md")

        lines = self.content.splitlines()
        sec_1_1_idx = None
        sec_1_2_idx = None
        for idx, line in enumerate(lines):
            if re.match(r'^###\s+1\.1\s+Primary Commercial Toolchain Integration', line):
                sec_1_1_idx = idx
            elif re.match(r'^###\s+1\.2\s+Three-Tier Architecture', line):
                sec_1_2_idx = idx

        self.assertIsNotNone(sec_1_1_idx, "### 1.1 Primary Commercial Toolchain Integration not found as H3")
        self.assertIsNotNone(sec_1_2_idx, "### 1.2 Three-Tier Architecture not found as H3")
        self.assertLess(
            sec_1_1_idx,
            sec_1_2_idx,
            f"Section 1.1 (line {sec_1_1_idx+1}) must precede Section 1.2 (line {sec_1_2_idx+1})"
        )

    def test_upstream_readme_three_tier_architecture_normalization(self):
        """Verifies that README.md cleanly defines three tiers and has 0 contradictory tier labels."""
        self.assertIn("### 1.2 Three-Tier Architecture & Repository Boundaries", self.content)
        self.assertIn("Tier 1: Upstream Specification Core Compiler", self.content)
        self.assertIn("Tier 2: Domain Distribution Templates", self.content)
        self.assertIn("Tier 3: Customer Application Workspaces", self.content)

        # Assert 0 contradictory duplicate Tier 1 labels for domain templates
        self.assertNotIn("Tier 1: Domain Distribution Templates", self.content)
        self.assertNotIn("Tier 1 Domain Distribution Template", self.content)

        # Assert 0 contradictory Tier 2 labels for customer workspaces
        self.assertNotIn("Tier 2: Customer Application Workspaces", self.content)
        self.assertNotIn("Tier 2 Customer Application Workspace", self.content)

    def test_upstream_readme_pipeline_2_prompt_boundary_confinement(self):
        """Verifies that Section 9.4 strictly confines Pipeline 2 prompts to DOWNSTREAM_CUSTOMER_PROJECT."""
        ast = parse_markdown_ast(self.content)
        sec9_4_nodes = ast.find_sections("9.4 Pipeline 2 Prompts")
        self.assertTrue(len(sec9_4_nodes) > 0, "Section 9.4 not found in README.md")

        sec9_4_text = sec9_4_nodes[0].get_all_text()
        self.assertIn("Execution Boundary Invariant", sec9_4_text)
        self.assertNotIn("(or UPSTREAM_SPEC_CORE_COMPILER depending on execution context)", sec9_4_text)
        self.assertNotIn("Repository Classification: UPSTREAM_SPEC_CORE_COMPILER", sec9_4_text)
        self.assertIn("Repository Classification: DOWNSTREAM_CUSTOMER_PROJECT", sec9_4_text)

    def test_upstream_readme_section_5_5_no_gemini_cli_conflation(self):
        """Verifies that Section 5.5 is dedicated to Google Antigravity and contains 0 occurrences of deprecated 'Gemini CLI' (Defect #379)."""
        ast = parse_markdown_ast(self.content)
        sec5_5_nodes = [s for s in ast.find_sections("5.5") if s.title.startswith("5.5")]
        self.assertEqual(len(sec5_5_nodes), 1, "Expected exactly one Section 5.5 in README.md AST")
        sec5_5_node = sec5_5_nodes[0]

        # Verify section title is dedicated to Google Antigravity without Gemini CLI conflation
        self.assertEqual(
            sec5_5_node.title.strip(),
            "5.5 Setup for Google Antigravity",
            f"Section 5.5 heading must be '5.5 Setup for Google Antigravity', got: '{sec5_5_node.title}'"
        )
        self.assertNotIn(
            "Gemini CLI",
            sec5_5_node.title,
            "Section 5.5 heading must not reference deprecated 'Gemini CLI'"
        )

        # Assert 0 occurrences of deprecated 'Gemini CLI' in Section 5.5 AST subtree
        sec5_5_text = sec5_5_node.get_all_text()
        self.assertNotIn(
            "Gemini CLI",
            sec5_5_text,
            "Found deprecated 'Gemini CLI' in Section 5.5 AST of README.md (Defect #379)"
        )

        # Also verify line-level slice between Section 5.5 and Section 5.6
        lines = self.content.splitlines()
        sec_5_5_idx = None
        sec_5_6_idx = None
        for idx, line in enumerate(lines):
            if re.match(r'^###\s+5\.5\s+', line):
                sec_5_5_idx = idx
            elif re.match(r'^###\s+5\.6\s+', line):
                sec_5_6_idx = idx

        self.assertIsNotNone(sec_5_5_idx, "Section 5.5 heading line not found in README.md")
        self.assertIsNotNone(sec_5_6_idx, "Section 5.6 heading line not found in README.md")
        self.assertLess(sec_5_5_idx, sec_5_6_idx, "Section 5.5 must precede Section 5.6")

        sec_5_5_slice = "\n".join(lines[sec_5_5_idx:sec_5_6_idx])
        self.assertNotIn(
            "Gemini CLI",
            sec_5_5_slice,
            "Found deprecated 'Gemini CLI' in Section 5.5 line slice of README.md (Defect #379)"
        )

    def test_upstream_readme_section_5_5_native_config_discovery_no_manual_chat_steering(self):
        """Verifies that Section 5.5 prescribes native configuration discovery and prohibits manual chat steering anti-patterns (Defect #380)."""
        ast = parse_markdown_ast(self.content)
        sec5_5_nodes = [s for s in ast.find_sections("5.5") if s.title.startswith("5.5")]
        self.assertEqual(len(sec5_5_nodes), 1, "Expected exactly one Section 5.5 in README.md AST")
        sec5_5_node = sec5_5_nodes[0]
        sec5_5_text = sec5_5_node.get_all_text()

        # 1. Negative Assertions: Prohibit manual chat steering anti-patterns
        banned_steering_phrases = [
            "Point Gemini",
            "Point Google Antigravity",
            "Point Antigravity",
            "Execute view_file directly on .pipeline/ACTIVE_RULES_BUNDLE.md, and read skills in",
            "Read the files in ./skills/, ./.agents/skills/, and ./rules/ directories.",
            "paste",
            "chat box",
            "chat prompt",
            "in chat",
        ]
        for phrase in banned_steering_phrases:
            self.assertNotIn(
                phrase.lower(),
                sec5_5_text.lower(),
                f"Found prohibited manual chat steering phrase '{phrase}' in Section 5.5 AST (Defect #380)"
            )

        # 2. Line-level slice check between Section 5.5 and Section 5.6
        lines = self.content.splitlines()
        sec_5_5_idx = None
        sec_5_6_idx = None
        for idx, line in enumerate(lines):
            if re.match(r'^###\s+5\.5\s+', line):
                sec_5_5_idx = idx
            elif re.match(r'^###\s+5\.6\s+', line):
                sec_5_6_idx = idx

        self.assertIsNotNone(sec_5_5_idx, "Section 5.5 heading line not found in README.md")
        self.assertIsNotNone(sec_5_6_idx, "Section 5.6 heading line not found in README.md")
        self.assertLess(sec_5_5_idx, sec_5_6_idx, "Section 5.5 must precede Section 5.6")

        sec_5_5_slice = "\n".join(lines[sec_5_5_idx:sec_5_6_idx])

        # Assert no text/code fences instructing manual chat prompt commands
        self.assertNotIn("```text\nExecute view_file", sec_5_5_slice)
        for phrase in banned_steering_phrases:
            self.assertNotIn(
                phrase.lower(),
                sec_5_5_slice.lower(),
                f"Found prohibited manual chat steering phrase '{phrase}' in Section 5.5 slice (Defect #380)"
            )

        # 3. Positive Assertions: Require affirmative native configuration discovery directives
        required_native_discovery_terms = [
            "Native Configuration Discovery",
            "AGENTS.md",
            ".agents/AGENTS.md",
            ".agents/skills/",
            "<user_rules>",
            "progressive disclosure",
            "automatically discovers",
            "CLAUDE.md",
            ".cursor/rules/",
        ]
        for term in required_native_discovery_terms:
            self.assertIn(
                term,
                sec_5_5_slice,
                f"Required native configuration discovery term '{term}' missing from Section 5.5 (Defect #380)"
            )

    def test_upstream_readme_section_5_universal_initialization_not_misplaced_in_vendor_5_5(self):
        """Verifies that the universal agent initialization sequence is placed in Section 5.6 and not misplaced inside vendor Section 5.5 (Defect #381)."""
        ast = parse_markdown_ast(self.content)

        # 1. Assert Section 5.5 heading is "5.5 Setup for Google Antigravity"
        sec5_5_nodes = [s for s in ast.find_sections("5.5") if s.title.startswith("5.5")]
        self.assertEqual(len(sec5_5_nodes), 1, "Expected exactly one Section 5.5 in README.md AST")
        sec5_5_node = sec5_5_nodes[0]
        self.assertEqual(
            sec5_5_node.title.strip(),
            "5.5 Setup for Google Antigravity",
            f"Section 5.5 heading must be '5.5 Setup for Google Antigravity', got: '{sec5_5_node.title}'"
        )

        # 2. Assert Section 5.5 does NOT contain the 8-step initialization sequence (steps 0 to 7) embedded in its AST or text slice
        steps_0_to_7 = [
            "Detect Repository Role & Scope",
            "Enforce Karpathy Compliance & Strict Planning Gate",
            "Read Governance Constitution",
            "Load Project Skills",
            "Load Governance Rules",
            "Load Platform Profile",
            "Bootstrap Tracker Labels",
            "Transition to Operator Prompt Catalog",
        ]
        sec5_5_text = sec5_5_node.get_all_text()
        for step in steps_0_to_7:
            self.assertNotIn(
                step,
                sec5_5_text,
                f"Universal initialization step '{step}' must not be embedded in Section 5.5 AST (Defect #381)"
            )

        lines = self.content.splitlines()
        sec_5_5_idx = None
        sec_5_6_idx = None
        sec_5_7_idx = None
        for idx, line in enumerate(lines):
            if re.match(r'^###\s+5\.5\s+', line):
                sec_5_5_idx = idx
            elif re.match(r'^###\s+5\.6\s+', line):
                sec_5_6_idx = idx
            elif re.match(r'^###\s+5\.7\s+', line):
                sec_5_7_idx = idx

        self.assertIsNotNone(sec_5_5_idx, "Section 5.5 heading line not found in README.md")
        self.assertIsNotNone(sec_5_6_idx, "Section 5.6 heading line not found in README.md")
        self.assertIsNotNone(sec_5_7_idx, "Section 5.7 heading line not found in README.md")
        self.assertLess(sec_5_5_idx, sec_5_6_idx, "Section 5.5 must precede Section 5.6")
        self.assertLess(sec_5_6_idx, sec_5_7_idx, "Section 5.6 must precede Section 5.7")

        sec_5_5_slice = "\n".join(lines[sec_5_5_idx:sec_5_6_idx])
        sec_5_6_slice = "\n".join(lines[sec_5_6_idx:sec_5_7_idx])
        for step in steps_0_to_7:
            self.assertNotIn(
                step,
                sec_5_5_slice,
                f"Universal initialization step '{step}' must not be embedded in Section 5.5 line slice (Defect #381)"
            )

        numbered_steps = [
            "0. **Detect Repository Role & Scope",
            "1. **Enforce Karpathy Compliance & Strict Planning Gate",
            "2. **Read Governance Constitution",
            "3. **Load Project Skills",
            "4. **Load Governance Rules",
            "5. **Load Platform Profile",
            "6. **Bootstrap Tracker Labels",
            "7. **Transition to Operator Prompt Catalog",
        ]
        for step in numbered_steps:
            self.assertNotIn(
                step,
                sec_5_5_slice,
                f"Numbered step '{step}' must not be in Section 5.5 line slice (Defect #381)"
            )
            self.assertIn(
                step,
                sec_5_6_slice,
                f"Numbered step '{step}' missing from Section 5.6 line slice (Defect #381)"
            )

        # 3. Assert Section 5.5 contains a reference to Section 5.6
        self.assertIn(
            "Section 5.6",
            sec5_5_text,
            "Section 5.5 AST must reference Section 5.6 (Defect #381)"
        )
        self.assertIn(
            "Section 5.6",
            sec_5_5_slice,
            "Section 5.5 line slice must reference Section 5.6 (Defect #381)"
        )

        # 4. Assert Section 5.6 heading is "5.6 Universal Agent Governance & Initialization Sequence (AGENTS.md)"
        sec5_6_nodes = [s for s in ast.find_sections("5.6") if s.title.startswith("5.6")]
        self.assertEqual(len(sec5_6_nodes), 1, "Expected exactly one Section 5.6 in README.md AST")
        sec5_6_node = sec5_6_nodes[0]
        self.assertEqual(
            sec5_6_node.title.strip(),
            "5.6 Universal Agent Governance & Initialization Sequence (AGENTS.md)",
            f"Section 5.6 heading must be '5.6 Universal Agent Governance & Initialization Sequence (AGENTS.md)', got: '{sec5_6_node.title}'"
        )

        # 5. Assert Section 5.6 AST contains all 8 steps (Steps 0 through 7)
        sec5_6_text = sec5_6_node.get_all_text()
        for step in steps_0_to_7:
            self.assertIn(
                step,
                sec5_6_text,
                f"Universal initialization step '{step}' missing from Section 5.6 AST (Defect #381)"
            )

        # 6. Assert Section 5.7 and Section 5.8 contain references to Section 5.6
        sec5_7_nodes = [s for s in ast.find_sections("5.7") if s.title.startswith("5.7")]
        self.assertEqual(len(sec5_7_nodes), 1, "Expected exactly one Section 5.7 in README.md AST")
        self.assertIn(
            "Section 5.6",
            sec5_7_nodes[0].get_all_text(),
            "Section 5.7 AST must reference Section 5.6 (Defect #381)"
        )

        sec5_8_nodes = [s for s in ast.find_sections("5.8") if s.title.startswith("5.8")]
        self.assertEqual(len(sec5_8_nodes), 1, "Expected exactly one Section 5.8 in README.md AST")
        self.assertIn(
            "Section 5.6",
            sec5_8_nodes[0].get_all_text(),
            "Section 5.8 AST must reference Section 5.6 (Defect #381)"
        )


class TestDomainDistributionTemplateScaffolding(unittest.TestCase):
    """Verifies that install_pipeline.sh properly scaffolds Tier 2 Domain Distribution Templates."""

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
            self.assertIn("As a **Tier 2 Domain Distribution Template**", content)
            self.assertNotIn("Tier 1 Domain Distribution Template", content)

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
    """Verifies that install_pipeline.sh properly scaffolds Tier 3 Customer Application Workspaces."""

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
            self.assertIn("As a **Tier 3 Customer Application Workspace**", content)
            self.assertNotIn("Tier 2 Customer Application Workspace", content)

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

    def test_customer_workspace_scaffolding_ast_section_4_1_topology(self):
        """Asserts Section 4.1 Master-Worker Subagent Topology contains Pipeline 1 and Pipeline 2 via CommonMark AST."""
        with tempfile.TemporaryDirectory() as temp_dir:
            proc = self._run_installer(temp_dir, "--role", "customer-project")
            self.assertEqual(proc.returncode, 0, f"Installer failed. stderr: {proc.stderr}")

            readme_path = os.path.join(temp_dir, "README.md")
            with open(readme_path, "r", encoding="utf-8") as f:
                content = f.read()

            ast = parse_markdown_ast(content)
            sec_4_1_nodes = ast.find_sections("4.1 Master-Worker Subagent Topology")
            self.assertTrue(
                len(sec_4_1_nodes) > 0,
                "Section '4.1 Master-Worker Subagent Topology' not found in customer README AST"
            )

            mermaid_fences = sec_4_1_nodes[0].find_fences("mermaid")
            self.assertTrue(
                len(mermaid_fences) > 0,
                "Section 4.1 must contain a Mermaid code block"
            )
            mermaid_body = mermaid_fences[0].content
            self.assertIn("Pipeline 0", mermaid_body)
            self.assertIn("Pipeline 1", mermaid_body)
            self.assertIn("Pipeline 2", mermaid_body)

    def test_customer_workspace_scaffolding_ast_section_4_5_autonomous_prompts(self):
        """Asserts Section 4.5 Pipeline 2 Autonomous Feature Implementation Prompts exists via CommonMark AST."""
        with tempfile.TemporaryDirectory() as temp_dir:
            proc = self._run_installer(temp_dir, "--role", "customer-project")
            self.assertEqual(proc.returncode, 0, f"Installer failed. stderr: {proc.stderr}")

            readme_path = os.path.join(temp_dir, "README.md")
            with open(readme_path, "r", encoding="utf-8") as f:
                content = f.read()

            ast = parse_markdown_ast(content)
            sec_4_5_nodes = ast.find_sections("4.5 Pipeline 2 Autonomous Feature Implementation Prompts")
            self.assertTrue(
                len(sec_4_5_nodes) > 0,
                "Section '4.5 Pipeline 2 Autonomous Feature Implementation Prompts' not found in customer README AST"
            )

    def test_customer_workspace_scaffolding_ast_worker_2a1_flutter_prompt(self):
        """Asserts Worker 2A.1 Flutter Feature Implementation Prompt exists with app_flutter/ and flutter analyze."""
        with tempfile.TemporaryDirectory() as temp_dir:
            proc = self._run_installer(temp_dir, "--role", "customer-project")
            self.assertEqual(proc.returncode, 0, f"Installer failed. stderr: {proc.stderr}")

            readme_path = os.path.join(temp_dir, "README.md")
            with open(readme_path, "r", encoding="utf-8") as f:
                content = f.read()

            ast = parse_markdown_ast(content)
            sec_nodes = ast.find_sections("Worker 2A.1 / Synthesis Driver: Flutter Feature Implementation Prompt")
            self.assertTrue(
                len(sec_nodes) > 0,
                "Section 'Worker 2A.1 / Synthesis Driver: Flutter Feature Implementation Prompt' not found in AST"
            )

            fences = sec_nodes[0].find_fences()
            self.assertTrue(len(fences) > 0, "Worker 2A.1 section must contain a code block")
            fence_content = fences[0].content

            # Explicit code blocks containing app_flutter/ and flutter analyze
            self.assertIn("app_flutter/", fence_content)
            self.assertIn("flutter analyze", fence_content)
            self.assertIn("flutter test", fence_content)
            self.assertIn("MVVM", fence_content)
            self.assertIn("The 15 Mandatory Domain Engineering Standards", fence_content)
            self.assertIn("---GOVERNANCE-END---", fence_content)

    def test_customer_workspace_scaffolding_ast_worker_2a2_target_platform_prompt(self):
        """Asserts Worker 2A.2 Target Platform Feature Implementation Prompt exists with target platform profile and execution bounds."""
        with tempfile.TemporaryDirectory() as temp_dir:
            proc = self._run_installer(temp_dir, "--role", "customer-project")
            self.assertEqual(proc.returncode, 0, f"Installer failed. stderr: {proc.stderr}")

            readme_path = os.path.join(temp_dir, "README.md")
            with open(readme_path, "r", encoding="utf-8") as f:
                content = f.read()

            ast = parse_markdown_ast(content)
            sec_nodes = ast.find_sections("Worker 2A.2 / Synthesis Driver: Target Platform Feature Implementation Prompt")
            self.assertTrue(
                len(sec_nodes) > 0,
                "Section 'Worker 2A.2 / Synthesis Driver: Target Platform Feature Implementation Prompt' not found in AST"
            )

            fences = sec_nodes[0].find_fences()
            self.assertTrue(len(fences) > 0, "Worker 2A.2 section must contain a code block")
            fence_content = fences[0].content

            self.assertIn("target_platform", fence_content)
            self.assertIn("Target Platform Execution Specialist", fence_content)
            self.assertIn("real-time execution bounds", fence_content)
            self.assertIn("---GOVERNANCE-END---", fence_content)

    def test_customer_workspace_scaffolding_schema_clean_landing_zone(self):
        """Asserts customer project scaffolding produces a pristine landing zone in schema/ with only .gitkeep (fixes #390)."""
        with tempfile.TemporaryDirectory() as temp_dir:
            cust_dir = os.path.join(temp_dir, "uav-011")
            os.makedirs(cust_dir)

            proc = self._run_installer(cust_dir, "--role", "customer-project")
            self.assertEqual(proc.returncode, 0, f"Installer failed. stderr: {proc.stderr}")

            schema_dir = os.path.join(cust_dir, "schema")
            self.assertTrue(os.path.isdir(schema_dir), "schema/ directory was not created in customer workspace")

            schema_entries = sorted(os.listdir(schema_dir))
            self.assertEqual(
                schema_entries,
                [".gitkeep"],
                f"Customer workspace schema/ must contain ONLY .gitkeep, found: {schema_entries}"
            )

            # Explicitly assert zero copied .sysml or .json files
            forbidden_files = [f for f in schema_entries if f.endswith(".sysml") or f.endswith(".json")]
            self.assertEqual(
                forbidden_files,
                [],
                f"Customer workspace schema/ contains forbidden .sysml or .json template files: {forbidden_files}"
            )

    def test_customer_workspace_scaffolding_schema_does_not_copy_installer_template_models(self):
        """Asserts installer never copies template .sysml or .json models into customer schema/ even if present in installer root (fixes #390)."""
        import shutil
        with tempfile.TemporaryDirectory() as temp_dir:
            mock_installer_dir = os.path.join(temp_dir, "mock_installer")
            os.makedirs(mock_installer_dir, exist_ok=True)
            for item in ("scripts", "rules", "skills", ".agents", ".pipeline", "tests"):
                src_item = os.path.join(PROJECT_ROOT, item)
                if os.path.isdir(src_item):
                    shutil.copytree(src_item, os.path.join(mock_installer_dir, item))
            for f in ("requirements.txt", "pyproject.toml", ".gitignore"):
                src_f = os.path.join(PROJECT_ROOT, f)
                if os.path.isfile(src_f):
                    shutil.copy(src_f, os.path.join(mock_installer_dir, f))

            # Plant template prototype models and domain configs in mock installer's schema/
            mock_schema = os.path.join(mock_installer_dir, "schema")
            os.makedirs(mock_schema, exist_ok=True)
            with open(os.path.join(mock_schema, "PROTOTYPE_MODEL.sysml"), "w", encoding="utf-8") as f:
                f.write("package PrototypeModel {}")
            with open(os.path.join(mock_schema, "domain_config.json"), "w", encoding="utf-8") as f:
                f.write('{"domain": "aviation", "system_identifier": "PROTOTYPE_01"}')
            with open(os.path.join(mock_schema, ".gitkeep"), "w", encoding="utf-8") as f:
                f.write("")

            os.makedirs(os.path.join(mock_installer_dir, ".pipeline", "upstream"), exist_ok=True)

            cust_target = os.path.join(temp_dir, "customer_app")
            cmd = ["bash", os.path.join(mock_installer_dir, "scripts", "install_pipeline.sh"), cust_target, "--role", "customer-project"]
            proc = subprocess.run(cmd, capture_output=True, text=True)
            self.assertEqual(proc.returncode, 0, f"Installer execution failed. stderr: {proc.stderr}\nstdout: {proc.stdout}")

            cust_schema = os.path.join(cust_target, "schema")
            self.assertTrue(os.path.isdir(cust_schema), "Customer workspace schema/ directory missing")
            entries = sorted(os.listdir(cust_schema))

            self.assertEqual(entries, [".gitkeep"], f"Customer workspace schema must only contain .gitkeep, found {entries}")
            self.assertFalse(os.path.exists(os.path.join(cust_schema, "PROTOTYPE_MODEL.sysml")), "PROTOTYPE_MODEL.sysml was copied into customer workspace")
            self.assertFalse(os.path.exists(os.path.join(cust_schema, "domain_config.json")), "domain_config.json was copied into customer workspace")


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

                    # Extract Section 4 (Operator Prompt Catalog) via CommonMark AST
                    ast = parse_markdown_ast(content)
                    sec4_nodes = ast.find_sections("4. Multi-Pipeline Operator Prompt Catalog")
                    self.assertTrue(len(sec4_nodes) > 0, f"Section 4 not found in {role} README AST")
                    sec4_text = sec4_nodes[0].get_all_text()

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

    def test_readme_upgrade_detection_missing_active_rules_bundle(self):
        """Verifies in-place upgrade re-scaffolds README when ACTIVE_RULES_BUNDLE.md is missing."""
        for role, dir_name in [("domain-template", "DEAP-uas-test"), ("customer-project", "uav-test")]:
            with self.subTest(role=role):
                with tempfile.TemporaryDirectory() as temp_dir:
                    proj_dir = os.path.join(temp_dir, dir_name)
                    os.makedirs(proj_dir)
                    role_tag = "DOMAIN_DISTRIBUTION_TEMPLATE" if role == "domain-template" else "DOWNSTREAM_CUSTOMER_PROJECT"
                    extra_header = "Customer Project Onboarding" if role == "domain-template" else "Project Lifecycle & Tooling Maintenance"
                    # Seed legacy README with catalog but without ACTIVE_RULES_BUNDLE.md
                    legacy_readme = (
                        f"# {dir_name} -- Downstream Cyber-Physical Infrastructure Safety Project\n\n"
                        f"> **Repository Role:** `{role_tag}`\n\n"
                        f"## 1. System Overview\n\nOverview\n\n"
                        f"## 2. Pipeline Structure & Governance\n\n- `rules/`\n\n"
                        f"## 3. {extra_header}\n\nInfo\n\n"
                        f"## 4. Multi-Pipeline Operator Prompt Catalog & Autonomous Execution Workflows\n\nPrompts\n"
                    )
                    readme_path = os.path.join(proj_dir, "README.md")
                    with open(readme_path, "w", encoding="utf-8") as f:
                        f.write(legacy_readme)

                    proc = self._run_installer(proj_dir, "--role", role)
                    self.assertEqual(proc.returncode, 0, f"Installer failed for {role}. stderr: {proc.stderr}")

                    with open(readme_path, "r", encoding="utf-8") as f:
                        upgraded = f.read()

                    self.assertIn("ACTIVE_RULES_BUNDLE.md", upgraded, f"README for {role} was not upgraded to include ACTIVE_RULES_BUNDLE.md")

    def test_readme_upgrade_detection_legacy_dual_track_rule(self):
        """Verifies in-place upgrade re-scaffolds README when legacy rules/dual-track-mbd-verification.md is present."""
        for role, dir_name in [("domain-template", "DEAP-uas-test"), ("customer-project", "uav-test")]:
            with self.subTest(role=role):
                with tempfile.TemporaryDirectory() as temp_dir:
                    proj_dir = os.path.join(temp_dir, dir_name)
                    os.makedirs(proj_dir)
                    role_tag = "DOMAIN_DISTRIBUTION_TEMPLATE" if role == "domain-template" else "DOWNSTREAM_CUSTOMER_PROJECT"
                    extra_header = "Customer Project Onboarding" if role == "domain-template" else "Project Lifecycle & Tooling Maintenance"
                    # Seed README that has ACTIVE_RULES_BUNDLE.md but also legacy isolated rule citation
                    legacy_readme = (
                        f"# {dir_name} -- Downstream Cyber-Physical Infrastructure Safety Project\n\n"
                        f"> **Repository Role:** `{role_tag}`\n\n"
                        f"## 1. System Overview\n\nOverview\n\n"
                        f"## 2. Pipeline Structure & Governance\n\n- `.pipeline/`: ACTIVE_RULES_BUNDLE.md\n\n"
                        f"## 3. {extra_header}\n\nInfo\n\n"
                        f"## 4. Multi-Pipeline Operator Prompt Catalog & Autonomous Execution Workflows\n\n"
                        f"Legacy prompt referencing rules/dual-track-mbd-verification.md\n"
                    )
                    readme_path = os.path.join(proj_dir, "README.md")
                    with open(readme_path, "w", encoding="utf-8") as f:
                        f.write(legacy_readme)

                    proc = self._run_installer(proj_dir, "--role", role)
                    self.assertEqual(proc.returncode, 0, f"Installer failed for {role}. stderr: {proc.stderr}")

                    with open(readme_path, "r", encoding="utf-8") as f:
                        upgraded = f.read()

                    self.assertNotIn("rules/dual-track-mbd-verification.md", upgraded, f"Legacy rule reference not purged from {role} README")
                    self.assertIn("ACTIVE_RULES_BUNDLE.md", upgraded)

    def test_customer_readme_title_deduplication(self):
        """Verifies that installer strips duplicate -- Downstream Systems Engineering Project and legacy suffixes."""
        with tempfile.TemporaryDirectory() as temp_dir:
            proj_dir = os.path.join(temp_dir, "uav-011")
            os.makedirs(proj_dir)

            # Seed README with duplicated title
            duplicated_title = "# uav-011 -- Downstream Cyber-Physical Infrastructure Safety Project -- Downstream Cyber-Physical Infrastructure Safety Project\n\n"
            body = (
                "> **Repository Role:** `DOWNSTREAM_CUSTOMER_PROJECT`\n\n"
                "## 1. System Overview\n\nOverview\n\n"
                "## 2. Pipeline Structure & Governance\n\n- `.pipeline/`: ACTIVE_RULES_BUNDLE.md\n\n"
                "## 3. Project Lifecycle & Tooling Maintenance\n\nInfo\n\n"
                "## 4. Multi-Pipeline Operator Prompt Catalog & Autonomous Execution Workflows\n\nPrompts\n"
            )
            readme_path = os.path.join(proj_dir, "README.md")
            with open(readme_path, "w", encoding="utf-8") as f:
                f.write(duplicated_title + body)

            proc = self._run_installer(proj_dir, "--role", "customer-project")
            self.assertEqual(proc.returncode, 0, f"Installer failed. stderr: {proc.stderr}")

            with open(readme_path, "r", encoding="utf-8") as f:
                first_line = f.readline().strip()

            expected_title = "# uav-011 -- Downstream Systems Engineering Project"
            self.assertEqual(first_line, expected_title, f"Title was not deduplicated. Got: {first_line}")


if __name__ == "__main__":
    unittest.main()

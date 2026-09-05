#!/usr/bin/env python3
"""
Unit test suite enforcing the Zero Em Dash Invariant across the repository.
Scans rules/, skills/, scripts/, .pipeline/, docs/, tests/, AGENTS.md,
.agents/AGENTS.md, README.md, and all tracked repository files to assert
zero occurrences of Unicode em dash (\\u2014).
"""

import os
import subprocess
import unittest

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
EM_DASH = chr(0x2014)


def _scan_directory(dir_path):
    """Recursively scan a directory for files containing em dashes."""
    violations = []
    if not os.path.isdir(dir_path):
        return violations

    for root, dirs, files in os.walk(dir_path):
        if ".git" in dirs:
            dirs.remove(".git")
        for f in files:
            file_path = os.path.join(root, f)
            try:
                with open(file_path, "r", encoding="utf-8") as fp:
                    for line_num, line in enumerate(fp, 1):
                        if EM_DASH in line:
                            rel_path = os.path.relpath(file_path, REPO_ROOT)
                            violations.append(f"{rel_path}:{line_num}: {line.strip()}")
            except (UnicodeDecodeError, PermissionError):
                pass
    return violations


def _scan_file(file_path):
    """Scan an individual file for em dashes."""
    violations = []
    if not os.path.isfile(file_path):
        return violations

    try:
        with open(file_path, "r", encoding="utf-8") as fp:
            for line_num, line in enumerate(fp, 1):
                if EM_DASH in line:
                    rel_path = os.path.relpath(file_path, REPO_ROOT)
                    violations.append(f"{rel_path}:{line_num}: {line.strip()}")
    except (UnicodeDecodeError, PermissionError):
        pass
    return violations


class TestNoEmDashIntegrity(unittest.TestCase):
    """Mechanical quality gate asserting zero Unicode em dash characters in the codebase."""

    def test_rules_no_emdash(self):
        """Assert zero em dashes in rules/ directory."""
        violations = _scan_directory(os.path.join(REPO_ROOT, "rules"))
        self.assertEqual(
            violations,
            [],
            f"Found {len(violations)} em dash violation(s) in rules/:\n" + "\n".join(violations),
        )

    def test_skills_no_emdash(self):
        """Assert zero em dashes in skills/ directory."""
        violations = _scan_directory(os.path.join(REPO_ROOT, "skills"))
        self.assertEqual(
            violations,
            [],
            f"Found {len(violations)} em dash violation(s) in skills/:\n" + "\n".join(violations),
        )

    def test_scripts_no_emdash(self):
        """Assert zero em dashes in scripts/ directory."""
        violations = _scan_directory(os.path.join(REPO_ROOT, "scripts"))
        self.assertEqual(
            violations,
            [],
            f"Found {len(violations)} em dash violation(s) in scripts/:\n" + "\n".join(violations),
        )

    def test_pipeline_no_emdash(self):
        """Assert zero em dashes in .pipeline/ directory."""
        violations = _scan_directory(os.path.join(REPO_ROOT, ".pipeline"))
        self.assertEqual(
            violations,
            [],
            f"Found {len(violations)} em dash violation(s) in .pipeline/:\n" + "\n".join(violations),
        )

    def test_docs_no_emdash(self):
        """Assert zero em dashes in docs/ directory."""
        violations = _scan_directory(os.path.join(REPO_ROOT, "docs"))
        self.assertEqual(
            violations,
            [],
            f"Found {len(violations)} em dash violation(s) in docs/:\n" + "\n".join(violations),
        )

    def test_tests_no_emdash(self):
        """Assert zero em dashes in tests/ directory."""
        violations = _scan_directory(os.path.join(REPO_ROOT, "tests"))
        self.assertEqual(
            violations,
            [],
            f"Found {len(violations)} em dash violation(s) in tests/:\n" + "\n".join(violations),
        )

    def test_agents_rules_no_emdash(self):
        """Assert zero em dashes in AGENTS.md and .agents/AGENTS.md."""
        violations = []
        violations.extend(_scan_file(os.path.join(REPO_ROOT, "AGENTS.md")))
        violations.extend(_scan_file(os.path.join(REPO_ROOT, ".agents", "AGENTS.md")))
        self.assertEqual(
            violations,
            [],
            f"Found {len(violations)} em dash violation(s) in AGENTS.md / .agents/AGENTS.md:\n"
            + "\n".join(violations),
        )

    def test_readme_no_emdash(self):
        """Assert zero em dashes in README.md."""
        violations = _scan_file(os.path.join(REPO_ROOT, "README.md"))
        self.assertEqual(
            violations,
            [],
            f"Found {len(violations)} em dash violation(s) in README.md:\n" + "\n".join(violations),
        )

    def test_all_tracked_files_no_emdash(self):
        """Assert zero em dashes across all tracked repository files."""
        res = subprocess.run(
            ["git", "ls-files"],
            cwd=REPO_ROOT,
            capture_output=True,
            text=True,
            check=True,
        )
        tracked_files = res.stdout.splitlines()
        violations = []
        for rel_path in tracked_files:
            file_path = os.path.join(REPO_ROOT, rel_path)
            if not os.path.isfile(file_path):
                continue
            violations.extend(_scan_file(file_path))

        self.assertEqual(
            violations,
            [],
            f"Found {len(violations)} em dash violation(s) across tracked files:\n"
            + "\n".join(violations),
        )


if __name__ == "__main__":
    unittest.main()

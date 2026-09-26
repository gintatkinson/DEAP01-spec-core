#!/usr/bin/env python3
# Copyright Gint Atkinson, gint.atkinson@gmail.com
"""
Unit and regression tests for Issue #375: Fail-Closed Baseline Gates (Checks 17, 20, 23).

Verifies that in downstream customer project workspaces:
1. Check 17 (Safety Integrity) fails closed (exit code 1) when docs/safety/ is missing or empty and allow_missing_specs=False.
2. Check 20 (WBS Suite) fails closed (exit code 1) when docs/management/WBS_DELIVERABLES_SUITE.md is missing and allow_missing_specs=False.
3. Check 23 (Factual Grounding) fails closed (exit code 1) when SysML model/schema ground truth is missing and allow_missing_specs=False.
4. Upstream compiler mode (with .pipeline/upstream/) preserves clean landing zones and exits cleanly.
5. allow_missing_specs=True allows nascent / bootstrapping projects to pass when not in strict mode.
"""

import os
import shutil
import sys
import tempfile
import unittest

repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if repo_root not in sys.path:
    sys.path.insert(0, repo_root)

scripts_dir = os.path.join(repo_root, "scripts")
if scripts_dir not in sys.path:
    sys.path.insert(0, scripts_dir)

from scripts.verify_downstream_baseline import (
    check_safety_integrity_and_sora_completeness,
    check_wbs_suite_integrity,
    check_factual_grounding,
)


class TestBaselineFailClosed(unittest.TestCase):
    def setUp(self):
        self.tmpdir = tempfile.mkdtemp()

    def tearDown(self):
        shutil.rmtree(self.tmpdir)

    def test_check17_missing_safety_dir_fails_closed_in_downstream_mode(self):
        """Check 17 must exit with code 1 when docs/safety/ is missing in downstream mode with allow_missing_specs=False."""
        with self.assertRaises(SystemExit) as ctx:
            check_safety_integrity_and_sora_completeness(self.tmpdir, allow_missing_specs=False)
        self.assertEqual(ctx.exception.code, 1)

    def test_check17_empty_safety_dir_fails_closed_in_downstream_mode(self):
        """Check 17 must exit with code 1 when docs/safety/ has no spec files in downstream mode with allow_missing_specs=False."""
        os.makedirs(os.path.join(self.tmpdir, "docs", "safety"), exist_ok=True)
        with self.assertRaises(SystemExit) as ctx:
            check_safety_integrity_and_sora_completeness(self.tmpdir, allow_missing_specs=False)
        self.assertEqual(ctx.exception.code, 1)

    def test_check17_allow_missing_specs_bypasses_in_downstream_mode(self):
        """Check 17 must return cleanly without SystemExit when allow_missing_specs=True."""
        # Should not raise SystemExit
        check_safety_integrity_and_sora_completeness(self.tmpdir, allow_missing_specs=True)

    def test_check17_upstream_clean_landing_zone_passes(self):
        """Check 17 must pass cleanly in upstream compiler mode with clean landing zone."""
        os.makedirs(os.path.join(self.tmpdir, ".pipeline", "upstream"), exist_ok=True)
        os.makedirs(os.path.join(self.tmpdir, "docs", "safety"), exist_ok=True)
        with open(os.path.join(self.tmpdir, "docs", "safety", ".gitkeep"), "w") as f:
            f.write("")
        # Should not raise SystemExit even with allow_missing_specs=False
        check_safety_integrity_and_sora_completeness(self.tmpdir, allow_missing_specs=False)

    def test_check20_missing_wbs_suite_fails_closed_in_downstream_mode(self):
        """Check 20 must exit with code 1 when WBS_DELIVERABLES_SUITE.md is missing in downstream mode with allow_missing_specs=False."""
        with self.assertRaises(SystemExit) as ctx:
            check_wbs_suite_integrity(self.tmpdir, allow_missing_specs=False)
        self.assertEqual(ctx.exception.code, 1)

    def test_check20_allow_missing_specs_bypasses_in_downstream_mode(self):
        """Check 20 must return cleanly without SystemExit when allow_missing_specs=True."""
        check_wbs_suite_integrity(self.tmpdir, allow_missing_specs=True)

    def test_check20_upstream_clean_landing_zone_passes(self):
        """Check 20 must pass cleanly in upstream compiler mode with clean landing zone."""
        os.makedirs(os.path.join(self.tmpdir, ".pipeline", "upstream"), exist_ok=True)
        # Should not raise SystemExit even with allow_missing_specs=False
        check_wbs_suite_integrity(self.tmpdir, allow_missing_specs=False)

    def test_check23_missing_sysml_model_fails_closed_in_downstream_mode(self):
        """Check 23 must exit with code 1 when SysML model/schema ground truth is missing in downstream mode with allow_missing_specs=False."""
        with self.assertRaises(SystemExit) as ctx:
            check_factual_grounding(self.tmpdir, allow_missing_specs=False)
        self.assertEqual(ctx.exception.code, 1)

    def test_check23_allow_missing_specs_bypasses_in_downstream_mode(self):
        """Check 23 must return cleanly without SystemExit when allow_missing_specs=True."""
        check_factual_grounding(self.tmpdir, allow_missing_specs=True)

    def test_check23_upstream_clean_landing_zone_passes(self):
        """Check 23 must pass cleanly in upstream compiler mode with clean landing zone."""
        os.makedirs(os.path.join(self.tmpdir, ".pipeline", "upstream"), exist_ok=True)
        # Should not raise SystemExit even with allow_missing_specs=False
        check_factual_grounding(self.tmpdir, allow_missing_specs=False)


if __name__ == "__main__":
    unittest.main()

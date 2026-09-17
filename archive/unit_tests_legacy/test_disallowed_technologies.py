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
from parity_auditor.validators.spec_validator import SpecValidator


class TestDisallowedTechnologies(unittest.TestCase):
    def test_clean_workspace_no_disallowed_tech(self):
        """Verify that workspace without disallowed technologies passes."""
        with tempfile.TemporaryDirectory() as tmpdir:
            os.makedirs(os.path.join(tmpdir, "docs", "features"), exist_ok=True)
            with open(os.path.join(tmpdir, "docs", "features", "feat-01.md"), "w", encoding="utf-8") as f:
                f.write("# Feature 01\n\nStandard pure C++ implementation.\n")

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = SpecValidator()
            errors = validator.validate(repo)
            self.assertEqual(errors, [])

    def test_disallowed_technology_from_profile_detected(self):
        """Verify that buzzwords forbidden in profile are flagged in specs."""
        with tempfile.TemporaryDirectory() as tmpdir:
            profiles_dir = os.path.join(tmpdir, ".pipeline", "profiles")
            features_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(profiles_dir, exist_ok=True)
            os.makedirs(features_dir, exist_ok=True)

            # Profile forbidding CUDA and PyTorch
            profile_content = """---
title: "Embedded Profile"
platform: embedded_rt
disallowed_technologies:
  - CUDA
  - PyTorch
---
# Embedded Profile
"""
            with open(os.path.join(profiles_dir, "embedded.md"), "w", encoding="utf-8") as f:
                f.write(profile_content)

            # Feature spec mentioning CUDA
            spec_content = """# Feature 01
This module uses CUDA acceleration for matrix operations.
"""
            with open(os.path.join(features_dir, "feat-01.md"), "w", encoding="utf-8") as f:
                f.write(spec_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = SpecValidator()
            errors = validator.validate(repo)

            self.assertEqual(len(errors), 1)
            self.assertEqual(errors[0].rule_id, "profile-disallowed-technology")
            self.assertIn("CUDA", str(errors[0]))
            self.assertIn("embedded.md", str(errors[0]))

    def test_disallowed_technology_from_harness_config(self):
        """Verify that disallowed technologies from deap_harness_config.yaml are detected."""
        with tempfile.TemporaryDirectory() as tmpdir:
            features_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(features_dir, exist_ok=True)

            # deap_harness_config.yaml
            harness_cfg = """disallowed_technologies:
  - TensorFlow
  - CUDA
"""
            with open(os.path.join(tmpdir, "deap_harness_config.yaml"), "w", encoding="utf-8") as f:
                f.write(harness_cfg)

            spec_content = """# Feature 02
Model inference executed via TensorFlow Lite runtime.
"""
            with open(os.path.join(features_dir, "feat-02.md"), "w", encoding="utf-8") as f:
                f.write(spec_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = SpecValidator()
            errors = validator.validate(repo)

            self.assertEqual(len(errors), 1)
            self.assertEqual(errors[0].rule_id, "profile-disallowed-technology")
            self.assertIn("TensorFlow", str(errors[0]))


if __name__ == "__main__":
    unittest.main()

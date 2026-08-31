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
from parity_auditor.validators.safety_trace_validator import (
    SafetyTraceValidator,
    _normalize_uca_id,
)


class TestSafetyTraceValidator(unittest.TestCase):
    def test_normalization_helper(self):
        """Verify UCA identifier canonical normalization."""
        self.assertEqual(_normalize_uca_id("UCA-01"), "UCA-01")
        self.assertEqual(_normalize_uca_id("UCA-1"), "UCA-01")
        self.assertEqual(_normalize_uca_id("Assert_UCA_01"), "UCA-01")
        self.assertEqual(_normalize_uca_id("Constraint_UCA_2"), "UCA-02")
        self.assertEqual(_normalize_uca_id("UCA_12"), "UCA-12")

    def test_clean_upstream_landing_zone_passes(self):
        """Verify that empty safety landing zone and schema passes gracefully."""
        with tempfile.TemporaryDirectory() as tmpdir:
            os.makedirs(os.path.join(tmpdir, "docs", "safety"), exist_ok=True)
            os.makedirs(os.path.join(tmpdir, "schema"), exist_ok=True)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = SafetyTraceValidator()
            errors = validator.validate(repo)
            self.assertEqual(errors, [])

    def test_exact_bidirectional_set_equality_passes(self):
        """Verify that matching set of UCAs across Markdown and SysML passes with zero errors."""
        with tempfile.TemporaryDirectory() as tmpdir:
            safety_dir = os.path.join(tmpdir, "docs", "safety")
            schema_dir = os.path.join(tmpdir, "schema")
            os.makedirs(safety_dir, exist_ok=True)
            os.makedirs(schema_dir, exist_ok=True)

            # Write STPA Matrix with UCA-01, UCA-02, UCA-03
            stpa_md = """# STPA Matrix
## Unsafe Control Actions
- **UCA-01**: Controller fails to issue command under critical condition.
- **UCA-02**: Controller issues command incorrectly.
- **UCA-03**: Controller applies command too long.
"""
            with open(os.path.join(safety_dir, "STPA_MATRIX.md"), "w", encoding="utf-8") as f:
                f.write(stpa_md)

            # Write SysML model with Assert_UCA_01, Assert_UCA_02, Assert_UCA_03
            sysml_content = """package SafetySSOT {
    assert constraint Assert_UCA_01 { /* Constraint for UCA-01 */ }
    assert constraint Assert_UCA_02 { /* Constraint for UCA-02 */ }
    assert constraint Assert_UCA_03 { /* Constraint for UCA-03 */ }
}
"""
            with open(os.path.join(schema_dir, "safety.sysml"), "w", encoding="utf-8") as f:
                f.write(sysml_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = SafetyTraceValidator()
            errors = validator.validate(repo)
            self.assertEqual(errors, [])

    def test_missing_in_sysml_detected(self):
        """Verify that UCA defined in Markdown but missing in SysML is flagged."""
        with tempfile.TemporaryDirectory() as tmpdir:
            safety_dir = os.path.join(tmpdir, "docs", "safety")
            schema_dir = os.path.join(tmpdir, "schema")
            os.makedirs(safety_dir, exist_ok=True)
            os.makedirs(schema_dir, exist_ok=True)

            # Markdown has UCA-01 and UCA-02
            stpa_md = """# STPA Matrix
- **UCA-01**: Hazard 1
- **UCA-02**: Hazard 2
"""
            with open(os.path.join(safety_dir, "STPA_MATRIX.md"), "w", encoding="utf-8") as f:
                f.write(stpa_md)

            # SysML only has Assert_UCA_01 (missing UCA-02)
            sysml_content = """package SafetySSOT {
    assert constraint Assert_UCA_01 { /* Constraint for UCA-01 */ }
}
"""
            with open(os.path.join(schema_dir, "safety.sysml"), "w", encoding="utf-8") as f:
                f.write(sysml_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = SafetyTraceValidator()
            errors = validator.validate(repo)

            self.assertEqual(len(errors), 1)
            self.assertEqual(errors[0].rule_id, "safety-trace-uca-missing-in-sysml")
            self.assertIn("UCA-02", str(errors[0]))

    def test_missing_in_markdown_detected(self):
        """Verify that UCA asserted in SysML but missing in Markdown is flagged."""
        with tempfile.TemporaryDirectory() as tmpdir:
            safety_dir = os.path.join(tmpdir, "docs", "safety")
            schema_dir = os.path.join(tmpdir, "schema")
            os.makedirs(safety_dir, exist_ok=True)
            os.makedirs(schema_dir, exist_ok=True)

            # Markdown only has UCA-01
            stpa_md = """# STPA Matrix
- **UCA-01**: Hazard 1
"""
            with open(os.path.join(safety_dir, "STPA_MATRIX.md"), "w", encoding="utf-8") as f:
                f.write(stpa_md)

            # SysML has Assert_UCA_01 and Assert_UCA_02 (extra UCA-02)
            sysml_content = """package SafetySSOT {
    assert constraint Assert_UCA_01 { /* Constraint for UCA-01 */ }
    assert constraint Assert_UCA_02 { /* Constraint for UCA-02 */ }
}
"""
            with open(os.path.join(schema_dir, "safety.sysml"), "w", encoding="utf-8") as f:
                f.write(sysml_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = SafetyTraceValidator()
            errors = validator.validate(repo)

            self.assertEqual(len(errors), 1)
            self.assertEqual(errors[0].rule_id, "safety-trace-uca-missing-in-markdown")
            self.assertIn("UCA-02", str(errors[0]))


if __name__ == "__main__":
    unittest.main()

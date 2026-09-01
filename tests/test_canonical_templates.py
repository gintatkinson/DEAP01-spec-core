import os
import re
import sys
import unittest

repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if repo_root not in sys.path:
    sys.path.insert(0, repo_root)

parity_src = os.path.join(repo_root, "skills", "spec-orchestrator", "parity_auditor", "src")
if parity_src not in sys.path:
    sys.path.insert(0, parity_src)

from parity_auditor.validators.conops_completeness_validator import (
    ConopsCompletenessValidator,
    MissionIntentCompletenessValidator,
)
from scripts.verify_downstream_baseline import check_upstream_template_clean_landing_zones


FORBIDDEN_DOMAIN_NOUNS = [
    r"\buav\b",
    r"\buas\b",
    r"\bdrone\b",
    r"\bdrones\b",
    r"\baircraft\b",
    r"\bairplane\b",
    r"\bquadcopter\b",
    r"\bmultirotor\b",
    r"\bfixed-wing\b",
    r"\bpropeller\b",
    r"\bpropellers\b",
    r"\bairframe\b",
    r"\bruddervator\b",
    r"\belevon\b",
    r"\belevons\b",
    r"\baileron\b",
    r"\bailerons\b",
    r"\bflight\b",
    r"\btakeoff\b",
    r"\btouchdown\b",
    r"\bparachute\b",
    r"\bloiter\b",
    r"\bpilot\b",
    r"\bcockpit\b",
    r"\bavenger\b",
    r"\bskyranger\b",
]


class TestCanonicalTemplates(unittest.TestCase):
    """
    Test suite for Issue #67: Permanent Abstract Templates.
    Verifies that CONOPS and Mission Intent canonical templates exist,
    parse cleanly via Gate 26, contain zero domain nouns, use parameter tokens {{...}},
    are mirrored in .agents/, and that docs/conops/ is a clean landing zone with only .gitkeep.
    """

    def setUp(self):
        self.conops_src = os.path.join(repo_root, "skills", "spec-orchestrator", "resources", "CONOPS_CANONICAL_TEMPLATE.md")
        self.mission_src = os.path.join(repo_root, "skills", "spec-orchestrator", "resources", "MISSION_INTENT_CANONICAL_TEMPLATE.md")
        self.conops_mirror = os.path.join(repo_root, ".agents", "skills", "spec-orchestrator", "resources", "CONOPS_CANONICAL_TEMPLATE.md")
        self.mission_mirror = os.path.join(repo_root, ".agents", "skills", "spec-orchestrator", "resources", "MISSION_INTENT_CANONICAL_TEMPLATE.md")

    def test_canonical_templates_exist_and_mirrored(self):
        """Verify that both canonical templates exist in resources/ and are mirrored in .agents/."""
        self.assertTrue(os.path.isfile(self.conops_src), f"Missing canonical template: {self.conops_src}")
        self.assertTrue(os.path.isfile(self.mission_src), f"Missing canonical template: {self.mission_src}")
        self.assertTrue(os.path.isfile(self.conops_mirror), f"Missing mirrored template: {self.conops_mirror}")
        self.assertTrue(os.path.isfile(self.mission_mirror), f"Missing mirrored template: {self.mission_mirror}")

        with open(self.conops_src, "r", encoding="utf-8") as f1, open(self.conops_mirror, "r", encoding="utf-8") as f2:
            self.assertEqual(f1.read(), f2.read(), "CONOPS template mirror mismatch")

        with open(self.mission_src, "r", encoding="utf-8") as f1, open(self.mission_mirror, "r", encoding="utf-8") as f2:
            self.assertEqual(f1.read(), f2.read(), "MISSION_INTENT template mirror mismatch")

    def test_conops_template_parses_cleanly_via_gate26(self):
        """Verify that CONOPS_CANONICAL_TEMPLATE.md parses with 0 errors via Gate 26 ConopsCompletenessValidator."""
        self.assertTrue(os.path.isfile(self.conops_src), f"Missing {self.conops_src}")
        with open(self.conops_src, "r", encoding="utf-8") as f:
            content = f.read()

        validator = ConopsCompletenessValidator()
        findings = validator._validate_conops_text(content, "CONOPS_CANONICAL_TEMPLATE.md")
        self.assertEqual(findings, [], f"Gate 26 findings on CONOPS template: {findings}")

    def test_mission_intent_template_parses_cleanly_via_gate26(self):
        """Verify that MISSION_INTENT_CANONICAL_TEMPLATE.md parses with 0 errors via Gate 26 MissionIntentCompletenessValidator."""
        self.assertTrue(os.path.isfile(self.mission_src), f"Missing {self.mission_src}")
        with open(self.mission_src, "r", encoding="utf-8") as f:
            content = f.read()

        validator = MissionIntentCompletenessValidator()
        findings = validator._validate_mission_text(content, "MISSION_INTENT_CANONICAL_TEMPLATE.md")
        self.assertEqual(findings, [], f"Gate 26 findings on MISSION_INTENT template: {findings}")

    def test_conops_template_zero_domain_nouns(self):
        """Verify that CONOPS_CANONICAL_TEMPLATE.md contains zero concrete domain nouns."""
        self.assertTrue(os.path.isfile(self.conops_src), f"Missing {self.conops_src}")
        with open(self.conops_src, "r", encoding="utf-8") as f:
            content = f.read()

        violations = []
        for pat in FORBIDDEN_DOMAIN_NOUNS:
            matches = re.findall(pat, content, re.IGNORECASE)
            if matches:
                violations.append(f"Found forbidden domain noun '{matches[0]}' matching pattern '{pat}'")

        self.assertEqual(violations, [], f"Forbidden domain nouns found in CONOPS template: {violations}")

    def test_mission_intent_template_zero_domain_nouns(self):
        """Verify that MISSION_INTENT_CANONICAL_TEMPLATE.md contains zero concrete domain nouns."""
        self.assertTrue(os.path.isfile(self.mission_src), f"Missing {self.mission_src}")
        with open(self.mission_src, "r", encoding="utf-8") as f:
            content = f.read()

        violations = []
        for pat in FORBIDDEN_DOMAIN_NOUNS:
            matches = re.findall(pat, content, re.IGNORECASE)
            if matches:
                violations.append(f"Found forbidden domain noun '{matches[0]}' matching pattern '{pat}'")

        self.assertEqual(violations, [], f"Forbidden domain nouns found in MISSION_INTENT template: {violations}")

    def test_templates_use_parameter_tokens(self):
        """Verify that both templates use parameter tokens in format {{...}}."""
        self.assertTrue(os.path.isfile(self.conops_src), f"Missing {self.conops_src}")
        self.assertTrue(os.path.isfile(self.mission_src), f"Missing {self.mission_src}")

        with open(self.conops_src, "r", encoding="utf-8") as f:
            c_content = f.read()
        with open(self.mission_src, "r", encoding="utf-8") as f:
            m_content = f.read()

        c_tokens = re.findall(r'\{\{[A-Za-z0-9_]+(?::[^\}]*)?\}\}', c_content)
        m_tokens = re.findall(r'\{\{[A-Za-z0-9_]+(?::[^\}]*)?\}\}', m_content)

        self.assertGreaterEqual(len(c_tokens), 10, f"Expected >= 10 tokens in CONOPS template, found {len(c_tokens)}")
        self.assertGreaterEqual(len(m_tokens), 10, f"Expected >= 10 tokens in MISSION_INTENT template, found {len(m_tokens)}")

    def test_docs_conops_landing_zone_contains_only_gitkeep(self):
        """Verify that docs/conops/ directory contains ONLY .gitkeep or README.md."""
        conops_dir = os.path.join(repo_root, "docs", "conops")
        self.assertTrue(os.path.isdir(conops_dir), f"docs/conops directory does not exist at {conops_dir}")

        files = os.listdir(conops_dir)
        self.assertTrue(set(files).issubset({".gitkeep", "README.md"}), f"docs/conops/ should contain ONLY .gitkeep / README.md, but found: {files}")

    def test_check16_clean_landing_zone_passes(self):
        """Verify that Check 16 (Upstream Template Clean Landing Zone Gate) passes."""
        # Should not raise SystemExit
        check_upstream_template_clean_landing_zones(repo_root)


if __name__ == "__main__":
    unittest.main()

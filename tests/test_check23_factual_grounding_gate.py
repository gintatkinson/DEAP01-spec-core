"""
Unit tests for Check 23 (Factual Grounding & Numeric Provenance Gate) and FactualGroundingValidator.

Remediates Issue #286: Eliminates synthetic in-memory string mocks, wiring all tests
directly to persistent fixtures under tests/fixtures/safety/.

Verifies:
1. Registration in AGGREGATING_VALIDATORS and scripts/verify_downstream_baseline.py Check 23 integration.
2. test_check23_rejects_ungrounded_vtail_with_4_ruddervators: Rejection of ungrounded V-tail claims when BOM specifies 4 ruddervators ('factual-grounding-numeric-drift').
3. test_check23_rejects_fabricated_gload_claims: Rejection of fabricated 15-20g G-load claims without OEM source ('factual-grounding-numeric-drift').
4. test_check23_rejects_unverified_stanag_protocol: Rejection of ungrounded/undeclared STANAG protocol claims ('factual-grounding-unverified-protocol').
5. test_check23_rejects_autonomous_arming_sequence: Rejection of autonomous arming sequences without prior human operator C2 consent ('factual-grounding-temporal-safety-violation').
6. test_check23_accepts_fully_grounded_specifications_with_citations: Acceptance of grounded parameters, declared protocols, and explicit SSOT citations.
7. test_check23_upstream_clean_landing_zone: Graceful zero findings on clean landing zones (empty schema/ or .gitkeep).
8. test_check23_baseline_verification_integration: Integration verification for check_factual_grounding and check_factual_grounding_and_provenance in verify_downstream_baseline.py.
9. test_check23_rejects_citation_fraud_with_system_exit: Rejection of citation fraud with SystemExit(1).
10. test_check23_rejects_ungrounded_dshot_protocol: Rejection of ungrounded DShot600 digital ESC protocol.
11. test_check23_rejects_ungrounded_execution_rates: Rejection of ungrounded 400 Hz inner loop rate.
12. test_check23_rejects_epistemic_exemption_bypass: Rejection of ungrounded claims even with epistemic tags (#378, #376).
13. test_check23_rejects_ungrounded_dshot_with_section_symbol_citation: Rejection of ungrounded DShot600 even when section symbol is on line.
14. test_check23_rejects_missing_file_citation: Rejection of missing file citation.
15. test_check23_rejects_uncited_katex_math_expressions_issue288: Rejection of uncited KaTeX expressions.
16. test_check23_accepts_katex_math_expressions_with_ssot_citation_issue288: Acceptance of cited KaTeX expressions.
17. test_check23_accepts_katex_math_expressions_declared_in_schema_issue288: Acceptance of schema-declared KaTeX expressions.
"""

import os
import shutil
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
from parity_auditor.validators.factual_grounding_validator import FactualGroundingValidator
from parity_auditor.aggregator import AGGREGATING_VALIDATORS
from scripts.verify_downstream_baseline import (
    check_factual_grounding,
    check_factual_grounding_and_provenance,
)

FIXTURES_DIR = os.path.join(repo_root, "tests", "fixtures", "safety")


class TestCheck23FactualGroundingGate(unittest.TestCase):
    def setUp(self):
        self.validator = FactualGroundingValidator()
        self.assertTrue(os.path.isdir(FIXTURES_DIR), f"Fixtures directory missing: {FIXTURES_DIR}")

    def test_registered_in_aggregating_validators(self):
        """Verify FactualGroundingValidator is registered in AGGREGATING_VALIDATORS."""
        self.assertIn(FactualGroundingValidator, AGGREGATING_VALIDATORS)

    def test_check23_rejects_ungrounded_vtail_with_4_ruddervators(self):
        """Test rejection of ungrounded V-tail claims when BOM/SysML specifies 4 ruddervators (X-tail)."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            shutil.copy(
                os.path.join(FIXTURES_DIR, "ground_truth_model.sysml"),
                os.path.join(schema_dir, "model.sysml"),
            )
            shutil.copy(
                os.path.join(FIXTURES_DIR, "ungrounded_vtail_conops.md"),
                os.path.join(docs_dir, "CONOPS_EMPENNAGE.md"),
            )

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])

            self.assertTrue(len(findings) >= 1)
            rule_ids = {f.rule_id for f in findings}
            self.assertIn("factual-grounding-numeric-drift", rule_ids)
            self.assertTrue(any("V-tail" in str(f) for f in findings))

    def test_check23_rejects_fabricated_gload_claims(self):
        """Test rejection of fabricated 15-20g G-load claims without OEM source."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema", "extracted")
            docs_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            shutil.copy(
                os.path.join(FIXTURES_DIR, "oem_bom_spec.md"),
                os.path.join(schema_dir, "oem_spec.md"),
            )
            shutil.copy(
                os.path.join(FIXTURES_DIR, "fabricated_gload_conops.md"),
                os.path.join(docs_dir, "CONOPS_LAUNCH.md"),
            )

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])

            self.assertTrue(len(findings) >= 1)
            rule_ids = {f.rule_id for f in findings}
            self.assertIn("factual-grounding-numeric-drift", rule_ids)
            self.assertTrue(any("15-20g" in str(f) for f in findings))

    def test_check23_rejects_unverified_stanag_protocol(self):
        """Test rejection of undeclared STANAG protocol claims not in schema."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "icds")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            shutil.copy(
                os.path.join(FIXTURES_DIR, "ground_truth_model.sysml"),
                os.path.join(schema_dir, "model.sysml"),
            )
            shutil.copy(
                os.path.join(FIXTURES_DIR, "unverified_protocol_icd.md"),
                os.path.join(docs_dir, "ICD_EXTERNAL_DATALINK.md"),
            )

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])

            self.assertTrue(len(findings) >= 1)
            rule_ids = {f.rule_id for f in findings}
            self.assertIn("factual-grounding-unverified-protocol", rule_ids)
            findings_text = " ".join([str(f) for f in findings])
            self.assertIn("STANAG 4586", findings_text)

    def test_check23_rejects_autonomous_arming_sequence(self):
        """Test rejection of autonomous arming sequence without human C2 command in Mermaid sequence diagram."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "use-cases")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            shutil.copy(
                os.path.join(FIXTURES_DIR, "ground_truth_model.sysml"),
                os.path.join(schema_dir, "model.sysml"),
            )
            shutil.copy(
                os.path.join(FIXTURES_DIR, "autonomous_arming_usecase.md"),
                os.path.join(docs_dir, "UC_01_ARMING.md"),
            )

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])

            self.assertTrue(len(findings) >= 1)
            rule_ids = {f.rule_id for f in findings}
            self.assertIn("factual-grounding-temporal-safety-violation", rule_ids)
            self.assertTrue(any("Autonomous arming sequence detected" in str(f) for f in findings))

    def test_check23_accepts_fully_grounded_specifications_with_citations(self):
        """Test acceptance of fully grounded specifications with valid SSOT citations and HITL sequence diagram."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "features")
            use_cases_dir = os.path.join(tmpdir, "docs", "use-cases")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)
            os.makedirs(use_cases_dir, exist_ok=True)

            shutil.copy(
                os.path.join(FIXTURES_DIR, "ground_truth_model.sysml"),
                os.path.join(schema_dir, "model.sysml"),
            )
            shutil.copy(
                os.path.join(FIXTURES_DIR, "fully_grounded_feature.md"),
                os.path.join(docs_dir, "FEAT_01_GROUNDED.md"),
            )
            shutil.copy(
                os.path.join(FIXTURES_DIR, "hitl_arming_usecase.md"),
                os.path.join(use_cases_dir, "UC_02_HITL_ARMING.md"),
            )

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])
            self.assertEqual(findings, [])

    def test_check23_upstream_clean_landing_zone(self):
        """Test acceptance of clean upstream landing zone (empty schema or .gitkeep)."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            with open(os.path.join(schema_dir, ".gitkeep"), "w", encoding="utf-8") as f:
                f.write("")

            with open(os.path.join(docs_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write("# Clean landing zone test\n")

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])
            self.assertEqual(findings, [])

    def test_check23_baseline_verification_integration(self):
        """Test check_factual_grounding_and_provenance and check_factual_grounding functions in clean and failing workspaces."""
        # 1. Clean workspace (upstream compiler landing zone)
        with tempfile.TemporaryDirectory() as tmpdir:
            upstream_marker = os.path.join(tmpdir, ".pipeline", "upstream")
            os.makedirs(upstream_marker, exist_ok=True)
            schema_dir = os.path.join(tmpdir, "schema")
            os.makedirs(schema_dir, exist_ok=True)
            with open(os.path.join(schema_dir, ".gitkeep"), "w", encoding="utf-8") as f:
                pass

            # Both alias and primary function should return cleanly without exit
            check_factual_grounding(tmpdir)
            check_factual_grounding_and_provenance(tmpdir)

        # 2. Failing workspace with ungrounded assertion
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            shutil.copy(
                os.path.join(FIXTURES_DIR, "ground_truth_model.sysml"),
                os.path.join(schema_dir, "model.sysml"),
            )
            shutil.copy(
                os.path.join(FIXTURES_DIR, "ungrounded_vtail_conops.md"),
                os.path.join(docs_dir, "CONOPS_EMPENNAGE.md"),
            )

            with self.assertRaises(SystemExit) as cm:
                check_factual_grounding(tmpdir)
            self.assertEqual(cm.exception.code, 1)

            with self.assertRaises(SystemExit) as cm_alias:
                check_factual_grounding_and_provenance(tmpdir)
            self.assertEqual(cm_alias.exception.code, 1)

    def test_check23_rejects_citation_fraud_with_system_exit(self):
        """Verify that Check 23 gate exits with code 1 when citation fraud is detected."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            shutil.copy(
                os.path.join(FIXTURES_DIR, "ground_truth_model.sysml"),
                os.path.join(schema_dir, "model.sysml"),
            )
            shutil.copy(
                os.path.join(FIXTURES_DIR, "oem_user_manual.md"),
                os.path.join(schema_dir, "oem_user_manual.md"),
            )
            shutil.copy(
                os.path.join(FIXTURES_DIR, "fraudulent_citation_feature.md"),
                os.path.join(docs_dir, "FEAT_SERVO.md"),
            )

            with self.assertRaises(SystemExit) as cm:
                check_factual_grounding(tmpdir)
            self.assertEqual(cm.exception.code, 1)

    def test_check23_rejects_ungrounded_dshot_protocol(self):
        """Verify that Check 23 gate exits with code 1 when ungrounded DShot600 ESC protocol is claimed."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            shutil.copy(
                os.path.join(FIXTURES_DIR, "ground_truth_model.sysml"),
                os.path.join(schema_dir, "model.sysml"),
            )
            shutil.copy(
                os.path.join(FIXTURES_DIR, "ungrounded_dshot_feature.md"),
                os.path.join(docs_dir, "FEAT_ESC.md"),
            )

            with self.assertRaises(SystemExit) as cm:
                check_factual_grounding(tmpdir)
            self.assertEqual(cm.exception.code, 1)

    def test_check23_rejects_ungrounded_execution_rates(self):
        """Verify that Check 23 gate exits with code 1 when ungrounded 400 Hz inner loop rate is claimed."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            shutil.copy(
                os.path.join(FIXTURES_DIR, "ground_truth_model.sysml"),
                os.path.join(schema_dir, "model.sysml"),
            )
            shutil.copy(
                os.path.join(FIXTURES_DIR, "ungrounded_rates_feature.md"),
                os.path.join(docs_dir, "FEAT_RATES.md"),
            )

            with self.assertRaises(SystemExit) as cm:
                check_factual_grounding(tmpdir)
            self.assertEqual(cm.exception.code, 1)

    def test_check23_rejects_epistemic_exemption_bypass(self):
        """Verify that Check 23 gate rejects ungrounded claims even when annotated with [TIER-3: DESIGN] (Issues #378, #376)."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            shutil.copy(
                os.path.join(FIXTURES_DIR, "ground_truth_model.sysml"),
                os.path.join(schema_dir, "model.sysml"),
            )
            shutil.copy(
                os.path.join(FIXTURES_DIR, "epistemic_exempt_feature.md"),
                os.path.join(docs_dir, "FEAT_EXEMPT.md"),
            )

            with self.assertRaises(SystemExit) as cm:
                check_factual_grounding(tmpdir)
            self.assertEqual(cm.exception.code, 1)

    def test_check23_rejects_ungrounded_dshot_with_section_symbol_citation(self):
        """Verify that Check 23 gate rejects ungrounded DShot600 protocol even when § section symbol is on line (Issue #286)."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            shutil.copy(
                os.path.join(FIXTURES_DIR, "ground_truth_model.sysml"),
                os.path.join(schema_dir, "model.sysml"),
            )
            shutil.copy(
                os.path.join(FIXTURES_DIR, "oem_user_manual.md"),
                os.path.join(schema_dir, "oem_user_manual.md"),
            )
            shutil.copy(
                os.path.join(FIXTURES_DIR, "section_symbol_fraud_feature.md"),
                os.path.join(docs_dir, "FEAT_ESC.md"),
            )

            with self.assertRaises(SystemExit) as cm:
                check_factual_grounding(tmpdir)
            self.assertEqual(cm.exception.code, 1)

    def test_check23_rejects_missing_file_citation(self):
        """Verify that Check 23 gate exits with code 1 when a cited file does not exist on disk (Issue #286)."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            shutil.copy(
                os.path.join(FIXTURES_DIR, "ground_truth_model.sysml"),
                os.path.join(schema_dir, "model.sysml"),
            )
            shutil.copy(
                os.path.join(FIXTURES_DIR, "missing_file_citation_feature.md"),
                os.path.join(docs_dir, "FEAT_MISSING.md"),
            )

            with self.assertRaises(SystemExit) as cm:
                check_factual_grounding(tmpdir)
            self.assertEqual(cm.exception.code, 1)

    def test_check23_rejects_uncited_katex_math_expressions_issue288(self):
        """Verify that KaTeX math expressions fail Check 23 gate with SystemExit(1)
        and factual-grounding-numeric-drift when uncited and not in schema (Issue #288).
        """
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            shutil.copy(
                os.path.join(FIXTURES_DIR, "ground_truth_model.sysml"),
                os.path.join(schema_dir, "model.sysml"),
            )
            shutil.copy(
                os.path.join(FIXTURES_DIR, "uncited_katex_feature.md"),
                os.path.join(docs_dir, "FEAT_KATEX.md"),
            )

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])
            drift_findings = [f for f in findings if f.rule_id == "factual-grounding-numeric-drift"]
            self.assertGreaterEqual(len(drift_findings), 4)
            findings_text = " ".join(str(f) for f in drift_findings)
            self.assertIn("13-14 bar", findings_text)
            self.assertIn("50 m", findings_text)
            self.assertIn("300 m", findings_text)
            self.assertIn("55 m/s", findings_text)

            with self.assertRaises(SystemExit) as cm:
                check_factual_grounding(tmpdir)
            self.assertEqual(cm.exception.code, 1)

    def test_check23_accepts_katex_math_expressions_with_ssot_citation_issue288(self):
        """Verify that KaTeX math expressions pass Check 23 gate cleanly
        when substantiated by an explicit verified SSOT citation (Issue #288).
        """
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            shutil.copy(
                os.path.join(FIXTURES_DIR, "ground_truth_model.sysml"),
                os.path.join(schema_dir, "model.sysml"),
            )
            shutil.copy(
                os.path.join(FIXTURES_DIR, "flight_manual_cited.md"),
                os.path.join(schema_dir, "flight-manual.md"),
            )
            shutil.copy(
                os.path.join(FIXTURES_DIR, "cited_katex_feature.md"),
                os.path.join(docs_dir, "FEAT_KATEX_CITED.md"),
            )

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])
            self.assertEqual(findings, [])

            # Check 23 gate must pass without SystemExit
            check_factual_grounding(tmpdir)

    def test_check23_accepts_katex_math_expressions_declared_in_schema_issue288(self):
        """Verify that KaTeX math expressions pass Check 23 gate cleanly
        when declared in schema ground truth limits (Issue #288).
        """
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            shutil.copy(
                os.path.join(FIXTURES_DIR, "grounded_limits_model.sysml"),
                os.path.join(schema_dir, "model.sysml"),
            )
            shutil.copy(
                os.path.join(FIXTURES_DIR, "grounded_katex_feature.md"),
                os.path.join(docs_dir, "FEAT_KATEX_GROUNDED.md"),
            )

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])
            drift_findings = [f for f in findings if f.rule_id == "factual-grounding-numeric-drift"]
            self.assertEqual(drift_findings, [])

            # Check 23 gate must pass without SystemExit
            check_factual_grounding(tmpdir)


if __name__ == "__main__":
    unittest.main()
